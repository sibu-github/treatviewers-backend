use mongodb::bson::doc;

use crate::{constants::*, models::*};

#[cfg_attr(test, mockall_double::double)]
use crate::config::database::DbClient;

pub struct WalletExtension;

#[cfg_attr(test, mockall::automock)]
impl WalletExtension {
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
}
