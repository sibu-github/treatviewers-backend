use mongodb::bson::doc;

use crate::{config::AppError, helpers::Helpers, import_double, models::*};

import_double!(DbClient);

pub async fn validate_add_bal_transaction(
    db: &DbClient,
    helper: &Helpers,
    user_id: u32,
    body: &AddBalEndReq,
) -> Result<(), AppError> {
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
