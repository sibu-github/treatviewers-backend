use std::sync::Arc;

use axum::{extract::State, Extension, Json};

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
