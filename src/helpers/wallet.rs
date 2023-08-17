use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    error::{Error as MongoError, Result as MongoResult},
    options::{FindOneAndUpdateOptions, ReturnDocument, UpdateModifications},
};

use crate::{config::AppError, constants::*, import_double, models::*, utils::get_epoch_ts};

import_double!(DbClient, DbSession);

pub struct WalletHelpers;

#[cfg_attr(test, mockall::automock)]
impl WalletHelpers {
    pub fn new() -> Self {
        Self
    }
    pub async fn get_user_balance(&self, db: &DbClient, user_id: u32) -> anyhow::Result<Money> {
        let filter = doc! {"userId": user_id};
        let wallet = db
            .find_one::<Wallet>(DB_NAME, COLL_WALLETS, Some(filter), None)
            .await?;
        let balance = wallet.and_then(|wallet| Some(wallet.balance()));
        Ok(balance.unwrap_or_default())
    }

    pub async fn get_wallet_transaction(
        &self,
        db: &DbClient,
        filter: Document,
    ) -> anyhow::Result<Option<WalletTransaction>> {
        let transaction = db
            .find_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, Some(filter), None)
            .await?;
        Ok(transaction)
    }

    pub async fn insert_wallet_transaction(
        &self,
        db: &DbClient,
        transaction: &WalletTransaction,
    ) -> anyhow::Result<String> {
        let inserted_id = db
            .insert_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, transaction, None)
            .await?;
        Ok(inserted_id.0)
    }

    pub async fn update_failed_transaction(
        &self,
        db: &DbClient,
        user_id: u32,
        transaction_id: &ObjectId,
        error_reason: &Option<String>,
        tracking_id: &Option<String>,
    ) -> anyhow::Result<()> {
        let filter = doc! {"_id": transaction_id};
        let ts = get_epoch_ts() as i64;
        let update = doc! {
            "$set": {
                "status": WalletTransactionStatus::Error.to_bson()?,
                "errorReason": error_reason,
                "trackingId": tracking_id,
                "updatedBy": user_id,
                "updatedTs": ts
            }
        };
        db.update_one(DB_NAME, COLL_WALLET_TRANSACTIONS, filter, update, None)
            .await?;
        Ok(())
    }

    pub async fn validate_add_bal_transaction(
        &self,
        db: &DbClient,
        user_id: u32,
        body: &AddBalEndReq,
    ) -> Result<(), AppError> {
        let filter = doc! {
            "_id": &body.transaction_id,
            "userId": user_id,
            "status": WalletTransactionStatus::Pending.to_bson()?,
            "transactionType": WalltetTransactionType::AddBalance.to_bson()?
        };
        let (transaction_result, balance_result) = tokio::join!(
            self.get_wallet_transaction(db, filter),
            self.get_user_balance(db, user_id)
        );
        let transaction =
            transaction_result?.ok_or(AppError::NotFound("transaction not found".into()))?;
        let user_balance = balance_result?;
        let amount = Money::new(body.amount, 0);
        if transaction.amount() != amount {
            let err = AppError::BadRequest("amount do not match".into());
            return Err(err);
        }
        if user_balance != transaction.balance_before() {
            let msg = format!(
                "user balance {} does not match with transaction balanceBefore {}",
                user_balance,
                transaction.balance_before()
            );
            let msg = Some(msg);
            self.update_failed_transaction(
                db,
                user_id,
                &body.transaction_id,
                &body.error_reason,
                &body.tracking_id,
            )
            .await?;
            let err = AppError::BadRequest(msg.unwrap());
            return Err(err);
        }
        Ok(())
    }

    pub async fn get_user_balance_session(
        &self,
        session: &mut DbSession,
        user_id: u32,
    ) -> MongoResult<Money> {
        let filter = doc! {"userId": user_id};
        let wallet = session
            .find_one_with_session::<Wallet>(DB_NAME, COLL_WALLETS, Some(filter), None)
            .await?;
        let balance = wallet
            .and_then(|wallet| Some(wallet.balance()))
            .unwrap_or_default();
        Ok(balance)
    }

    pub async fn update_wallet_with_session(
        &self,
        session: &mut DbSession,
        user_id: u32,
        real: u64,
        bonus: u64,
        subtract: bool,
        update_withdrawable: bool,
    ) -> MongoResult<(Money, Money)> {
        let balance_before = self.get_user_balance_session(session, user_id).await?;
        let withdrawable = if update_withdrawable { real } else { 0 };
        let wallet = if subtract {
            self.sub_wallet(session, user_id, real, bonus, withdrawable)
                .await?
        } else {
            self.add_wallet(session, user_id, real, bonus, withdrawable)
                .await?
        };
        let money = Money::new(real, bonus);
        let balance_after = if subtract {
            balance_before - money
        } else {
            balance_before + money
        };
        if wallet.balance() != balance_after {
            let err = format!("balance_before {:?} and balance_after {:?} not matching, required balance_after {:?}",
                balance_before,
                wallet.balance(),
                balance_after
            );
            let err = MongoError::custom(err);
            return Err(err);
        }
        Ok((balance_before, wallet.balance()))
    }

    async fn find_and_modify_wallet<U>(
        &self,
        session: &mut DbSession,
        filter: Document,
        update: U,
    ) -> MongoResult<Wallet>
    where
        U: Into<UpdateModifications> + 'static,
    {
        let options = FindOneAndUpdateOptions::builder()
            .upsert(Some(true))
            .return_document(Some(ReturnDocument::After))
            .build();
        let wallet = session
            .find_one_and_update_with_session::<Wallet, U>(
                DB_NAME,
                COLL_WALLETS,
                filter,
                update,
                Some(options),
            )
            .await?
            .ok_or(MongoError::custom("not able to update wallet"))?;
        Ok(wallet)
    }

    async fn add_wallet(
        &self,
        session: &mut DbSession,
        user_id: u32,
        real: u64,
        bonus: u64,
        withdrawable: u64,
    ) -> MongoResult<Wallet> {
        let filter = doc! {"userId": user_id};
        let ts = get_epoch_ts() as i64;
        let update = doc! {
            "$inc": {
                "balance.bonus": bonus as i64,
                "balance.real": real as i64,
                "balance.withdrawable": withdrawable as i64
            },
            "$setOnInsert": {"createdTs": ts},
            "$set": {"updatedTs": ts},
        };
        self.find_and_modify_wallet(session, filter, update).await
    }

    async fn sub_wallet(
        &self,
        session: &mut DbSession,
        user_id: u32,
        real: u64,
        bonus: u64,
        withdrawable: u64,
    ) -> MongoResult<Wallet> {
        let real = real as i64;
        let bonus = bonus as i64;
        let wd = withdrawable as i64;
        let ts = get_epoch_ts() as i64;
        let filter = doc! {
            "userId": user_id,
            "balance.real": {"$gte": real},
            "balance.bonus": {"$gte": bonus}
        };
        let update = vec![
            doc! {
                "$set": {
                    "balance.real": {
                        "$cond": [
                            {"$gte": ["$balance.real", real]},
                            {"$subtract": ["$balance.real", real]},
                            0
                        ]
                    },
                    "balance.bonus": {
                        "$cond": [
                            {"$gte": ["$balance.bonus", bonus]},
                            {"$subtract": ["$balance.bonus", bonus]},
                            0
                        ]
                    },
                    "balance.withdrawable": {
                        "$cond": [
                            {"$gte": ["$balance.withdrawable", wd]},
                            {"$subtract": ["$balance.withdrawable", wd]},
                            0
                        ]
                    },
                    "updatedTs": ts
                }
            },
            doc! {
                "$set": {
                    "balance.withdrawable": {
                        "$cond":[
                            {"$gt": ["$balance.withdrawable", "$balance.real"]},
                            "$balance.real",
                            "$balance.withdrawable"
                        ]
                    }
                }
            },
        ];
        self.find_and_modify_wallet(session, filter, update).await
    }

    pub async fn update_wallet_transaction_session(
        &self,
        session: &mut DbSession,
        transaction_id: &ObjectId,
        balance_after: Money,
        tracking_id: &Option<String>,
    ) -> MongoResult<()> {
        let ts = get_epoch_ts() as i64;
        let filter = doc! {"_id": transaction_id};
        let balance_after = balance_after
            .to_bson()
            .map_err(|e| MongoError::custom(e.to_string()))?;
        let status = WalletTransactionStatus::Completed
            .to_bson()
            .map_err(|e| MongoError::custom(e.to_string()))?;
        let update = doc! {
            "$set": {
                "balanceAfter": balance_after,
                "status": status,
                "trackingId": tracking_id,
                "updatedTs": ts
            }
        };
        session
            .update_one_with_session(DB_NAME, COLL_WALLET_TRANSACTIONS, filter, update, None)
            .await?;
        Ok(())
    }
}
