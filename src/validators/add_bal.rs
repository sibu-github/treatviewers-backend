use mongodb::bson::doc;

use crate::{config::AppError, helpers::Helpers, import_double, models::*};

import_double!(DbClient);

pub async fn validate_add_bal_transaction(
    db: &DbClient,
    helper: &Helpers,
    user_id: u32,
    body: &AddBalEndReq,
) -> Result<(), AppError> {
    if !body.is_successful {
        return Ok(());
    }
    let filter = doc! {
        "_id": &body.transaction_id,
        "userId": user_id,
        "status": WalletTransactionStatus::Pending.to_bson()?,
        "transactionType": WalltetTransactionType::AddBalance.to_bson()?
    };
    let wallet_helpers = helper.wallet_helpers();
    let (transaction_result, balance_result) = tokio::join!(
        wallet_helpers.get_wallet_transaction(db, filter),
        wallet_helpers.get_user_balance(db, user_id)
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
        wallet_helpers
            .update_failed_transaction(
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

#[cfg(test)]
mod tests {

    use anyhow::anyhow;
    use mockall::predicate::{eq, function};
    use mongodb::bson::oid::ObjectId;

    use crate::config::AppState;

    use super::*;

    #[tokio::test]
    async fn test_validate_add_bal_transaction() {
        let amount = 10;
        let mut req = AddBalEndReq {
            amount: amount,
            transaction_id: ObjectId::new(),
            is_successful: false,
            error_reason: None,
            tracking_id: None,
        };
        let user_id = 5;
        let mut state = AppState::mock();
        // scenario is_successful = false then this function returns Ok immediately
        let wallet_helpers = state.get_mut_helpers().mut_wallet_helpers();
        wallet_helpers.expect_get_wallet_transaction().never();
        wallet_helpers.expect_get_user_balance().never();
        let result = validate_add_bal_transaction(state.db(), state.helpers(), user_id, &req).await;
        assert_eq!(result.is_ok(), true);

        req.is_successful = true;

        // scenario - get_wallet_transaction function return Err
        let wallet_helpers = state.get_mut_helpers().mut_wallet_helpers();
        wallet_helpers
            .expect_get_wallet_transaction()
            .once()
            .returning(|_, _| Err(anyhow!("some error")));
        wallet_helpers
            .expect_get_user_balance()
            .once()
            .with(function(|db: &DbClient| true), eq(user_id))
            .returning(|_, _| Ok(Money::default()));
        let result = validate_add_bal_transaction(state.db(), state.helpers(), user_id, &req).await;
        match result {
            Err(AppError::AnyError(e)) => assert_eq!(e.to_string(), "some error"),
            _ => panic!(),
        };

        // scenario WalletTransaction not found
        let wallet_helpers = state.get_mut_helpers().mut_wallet_helpers();
        wallet_helpers
            .expect_get_wallet_transaction()
            .once()
            .returning(|_, _| Ok(None));
        wallet_helpers
            .expect_get_user_balance()
            .once()
            .with(function(|db: &DbClient| true), eq(user_id))
            .returning(|_, _| Ok(Money::default()));
        let result = validate_add_bal_transaction(state.db(), state.helpers(), user_id, &req).await;
        match result {
            Err(AppError::NotFound(e)) => assert_eq!(e, "transaction not found"),
            _ => panic!(),
        };

        // scenario get_user_balance return Err
        let wallet_helpers = state.get_mut_helpers().mut_wallet_helpers();
        wallet_helpers
            .expect_get_wallet_transaction()
            .once()
            .returning(|_, _| Ok(Some(WalletTransaction::default())));
        wallet_helpers
            .expect_get_user_balance()
            .once()
            .with(function(|db: &DbClient| true), eq(user_id))
            .returning(|_, _| Err(anyhow!("some error")));
        let result = validate_add_bal_transaction(state.db(), state.helpers(), user_id, &req).await;
        match result {
            Err(AppError::AnyError(e)) => assert_eq!(e.to_string(), "some error"),
            _ => panic!(),
        };

        // scenario amount do not match
        let wallet_helpers = state.get_mut_helpers().mut_wallet_helpers();
        wallet_helpers
            .expect_get_wallet_transaction()
            .once()
            .returning(|_, _| Ok(Some(WalletTransaction::default())));
        wallet_helpers
            .expect_get_user_balance()
            .once()
            .with(function(|db: &DbClient| true), eq(user_id))
            .returning(|_, _| Ok(Money::default()));
        wallet_helpers.expect_update_failed_transaction().never();
        let result = validate_add_bal_transaction(state.db(), state.helpers(), user_id, &req).await;
        match result {
            Err(AppError::BadRequest(e)) => assert_eq!(e.to_string(), "amount do not match"),
            _ => panic!(),
        };

        // scenario user balance do not match and error
        let transaction_id = req.transaction_id.clone();
        let error_reason = req.error_reason.clone();
        let tracking_id = req.tracking_id.clone();
        let wallet_helpers = state.get_mut_helpers().mut_wallet_helpers();
        wallet_helpers
            .expect_get_wallet_transaction()
            .once()
            .returning(move |_, _| {
                Ok(Some(WalletTransaction::add_bal_init_trans(
                    user_id,
                    amount,
                    Money::default(),
                )))
            });
        wallet_helpers
            .expect_get_user_balance()
            .once()
            .with(function(|db: &DbClient| true), eq(user_id))
            .returning(|_, _| Ok(Money::new(3, 0)));
        wallet_helpers
            .expect_update_failed_transaction()
            .once()
            .with(
                function(|db: &DbClient| true),
                eq(user_id),
                eq(transaction_id),
                eq(error_reason),
                eq(tracking_id),
            )
            .return_once(|_, _, _, _, _| Ok(()));
        let result = validate_add_bal_transaction(state.db(), state.helpers(), user_id, &req).await;
        match result {
            Err(AppError::BadRequest(e)) => assert_eq!(
                e.contains("does not match with transaction balanceBefore"),
                true
            ),
            _ => panic!(),
        };

        // user balance matched and success
        let wallet_helpers = state.get_mut_helpers().mut_wallet_helpers();
        wallet_helpers
            .expect_get_wallet_transaction()
            .once()
            .returning(move |_, _| {
                Ok(Some(WalletTransaction::add_bal_init_trans(
                    user_id,
                    amount,
                    Money::new(3, 0),
                )))
            });
        wallet_helpers
            .expect_get_user_balance()
            .once()
            .with(function(|db: &DbClient| true), eq(user_id))
            .returning(|_, _| Ok(Money::new(3, 0)));
        wallet_helpers.expect_update_failed_transaction().never();
        let result = validate_add_bal_transaction(state.db(), state.helpers(), user_id, &req).await;
        assert_eq!(result.is_ok(), true);
    }
}
