use std::sync::Arc;

use axum::{async_trait, extract::State, Extension, Json};
use futures::FutureExt;
use mongodb::bson::doc;

use crate::{
    config::{AppError, AppState, ValidatedBody},
    models::*,
    validators::ValidateExtra,
};

#[cfg_attr(test, mockall_double::double)]
use super::extension::WalletExtension;

/// Add balance initialize
///
/// Initialize add balance transaction
#[utoipa::path(
    post,
    path = "/api/v1/wallet/addBalanceInit",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = AddBalInitReq,
    responses(
        (status = StatusCode::OK, description = "Add balance initialized", body = AddBalInitRes),
    ),
    tag = "App User API"
)]
pub async fn add_bal_init_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Extension(extn): Extension<Arc<WalletExtension>>,
    ValidatedBody(body): ValidatedBody<AddBalInitReq>,
) -> Result<Json<AddBalInitRes>, AppError> {
    let app_upi_id = std::env::var("APP_UPI_ID")?;
    let balance_before = extn.get_user_balance(state.db(), claims.id).await?;
    let transaction = WalletTransaction::add_bal_init_trans(claims.id, body.amount, balance_before);
    let transaction_id = extn
        .insert_wallet_transaction(state.db(), &transaction)
        .await?;
    let res = AddBalInitRes {
        success: true,
        transaction_id,
        app_upi_id,
    };
    Ok(Json(res))
}

impl ValidateExtra for AddBalInitReq {}

#[async_trait]
impl ValidateExtra for AddBalEndReq {
    async fn validate_extra(&self, _state: Arc<AppState>) -> Result<(), AppError> {
        if !self.is_successful && self.error_reason.is_none() {
            let err = "errorReason is required for failed transaction";
            return Err(AppError::BadRequest(err.into()));
        }
        Ok(())
    }
}

/// Add balance finalize
///
/// Finalize add balance transaction
#[utoipa::path(
    post,
    path = "/api/v1/wallet/addBalanceEnd",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = AddBalEndReq,
    responses(
        (status = StatusCode::OK, description = "Add balance successful", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn add_bal_end_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Extension(extn): Extension<Arc<WalletExtension>>,
    ValidatedBody(body): ValidatedBody<AddBalEndReq>,
) -> Result<Json<GenericResponse>, AppError> {
    validate_transaction(&state, extn.as_ref(), claims.id, &body).await?;
    if body.is_successful {
        handle_success_transaction(&state, &extn, claims.id, &body).await
    } else {
        handle_failed_transaction(&state, &extn, claims.id, &body).await
    }
}

async fn handle_success_transaction(
    state: &Arc<AppState>,
    extn: &Arc<WalletExtension>,
    user_id: u32,
    body: &AddBalEndReq,
) -> Result<Json<GenericResponse>, AppError> {
    let extn = extn.clone();
    let transaction_id = body.transaction_id.clone();
    let tracking_id = body.tracking_id.clone();
    let amount = body.amount;
    state
        .db()
        .execute_transaction(None, None, move |session| {
            // TODO: currently all captured variables to be cloned twice.
            // Find a way to fix this problem.
            let extn = extn.clone();
            let transaction_id = transaction_id.clone();
            let tracking_id = tracking_id.clone();
            async move {
                let (_, balance_after) = extn
                    .update_wallet_with_session(session, user_id, amount, 0, false, false)
                    .await?;
                extn.update_wallet_transaction_session(
                    session,
                    &transaction_id,
                    balance_after,
                    &tracking_id,
                )
                .await?;

                Ok(())
            }
            .boxed()
        })
        .await?;

    Ok(GenericResponse::json_response(true, "Updated successfully"))
}

async fn handle_failed_transaction(
    state: &Arc<AppState>,
    extn: &WalletExtension,
    user_id: u32,
    body: &AddBalEndReq,
) -> Result<Json<GenericResponse>, AppError> {
    extn.updated_failed_transaction(
        state,
        user_id,
        &body.transaction_id,
        &body.error_reason,
        &body.tracking_id,
    )
    .await?;
    Ok(GenericResponse::json_response(true, "Updated successfully"))
}

async fn validate_transaction(
    state: &Arc<AppState>,
    extn: &WalletExtension,
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
        extn.get_wallet_transaction(state.db(), filter),
        extn.get_user_balance(state.db(), user_id)
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
        extn.updated_failed_transaction(
            state,
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
