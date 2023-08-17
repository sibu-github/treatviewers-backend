use std::sync::Arc;

use axum::{async_trait, extract::State, Extension, Json};
use futures::FutureExt;
use mongodb::bson::doc;

use crate::{
    config::{AppError, AppState, ValidatedBody},
    models::*,
    validators::ValidateExtra,
};

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
    ValidatedBody(body): ValidatedBody<AddBalInitReq>,
) -> Result<Json<AddBalInitRes>, AppError> {
    let app_upi_id = std::env::var("APP_UPI_ID")?;
    let wallet_helpers = state.helpers().wallet_helpers();
    let balance_before = wallet_helpers
        .get_user_balance(state.db(), claims.id)
        .await?;
    let transaction = WalletTransaction::add_bal_init_trans(claims.id, body.amount, balance_before);
    let transaction_id = wallet_helpers
        .insert_wallet_transaction(state.db(), &transaction)
        .await?;
    let res = AddBalInitRes {
        success: true,
        transaction_id,
        app_upi_id,
    };
    Ok(Json(res))
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
    ValidatedBody(body): ValidatedBody<AddBalEndReq>,
) -> Result<Json<GenericResponse>, AppError> {
    if body.is_successful {
        handle_success_transaction(state, claims.id, &body).await?;
    } else {
        handle_failed_transaction(&state, claims.id, &body).await?;
    }
    Ok(GenericResponse::json_response(true, "Updated successfully"))
}

async fn handle_success_transaction(
    state: Arc<AppState>,
    user_id: u32,
    body: &AddBalEndReq,
) -> Result<(), AppError> {
    let db = state.db();
    let cloned_state = state.clone();
    let transaction_id = body.transaction_id.clone();
    let tracking_id = body.tracking_id.clone();
    let amount = body.amount;
    db.execute_transaction(None, None, move |session| {
        // TODO: currently all captured variables to be cloned twice.
        // Find a way to fix this problem.
        let cloned_state = cloned_state.clone();
        let transaction_id = transaction_id.clone();
        let tracking_id = tracking_id.clone();
        async move {
            let wallet_helpers = cloned_state.helpers().wallet_helpers();
            let (_, balance_after) = wallet_helpers
                .update_wallet_with_session(session, user_id, amount, 0, false, false)
                .await?;
            wallet_helpers
                .update_wallet_transaction_session(
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

    Ok(())
}

async fn handle_failed_transaction(
    state: &Arc<AppState>,
    user_id: u32,
    body: &AddBalEndReq,
) -> Result<(), AppError> {
    state
        .helpers()
        .wallet_helpers()
        .update_failed_transaction(
            state.db(),
            user_id,
            &body.transaction_id,
            &body.error_reason,
            &body.tracking_id,
        )
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use mockall::predicate::{always, eq, function};
    use mongodb::bson::oid::ObjectId;
    use serde_json::json;

    use crate::{
        config::build_app_routes,
        helpers::Helpers,
        import_double,
        utils::{
            get_epoch_ts,
            test_helper::{build_post_request, oneshot_request},
        },
    };

    import_double!(DbClient);

    use super::*;

    #[tokio::test]
    async fn test_add_balance_end_handler_success_transaction() {
        let ts = get_epoch_ts();
        let token = "DUMMY_TOKEN";
        let user_id = 10;
        let amount = 10;
        let transaction_id = "6494fdba5155b267cb139995";
        let mut state = AppState::mock();
        state
            .get_mut_db()
            .expect_execute_transaction()
            .once()
            .with(
                function(Option::is_none),
                function(Option::is_none),
                always(),
            )
            .return_once(|_, _, _| Ok(()));
        state
            .get_mut_utility()
            .expect_decode_token()
            .once()
            .with(eq(token))
            .returning(move |_| Ok(JwtClaims::new(user_id, None, false, ts as usize)));
        state
            .get_mut_validators()
            .expect_validate_add_bal_transaction()
            .once()
            .with(
                function(|db: &DbClient| true),
                function(|h: &Helpers| true),
                eq(user_id),
                function(move |body: &AddBalEndReq| {
                    body.amount == amount
                        && body.is_successful
                        && body.transaction_id.to_hex().as_str() == transaction_id
                        && body.error_reason.is_none()
                        && body.tracking_id.is_none()
                }),
            )
            .return_once(|_, _, _, _| Ok(()));
        let state = Arc::new(state);
        let app = build_app_routes(state);
        let path = "/api/v1/wallet/addBalanceEnd";
        let body = json!({"amount": amount, "isSuccessful": true, "transactionId": transaction_id});
        let body = build_post_request(path, body.to_string().as_str(), Some(token));
        let res = oneshot_request::<GenericResponse>(app, body, Some(StatusCode::OK)).await;
        assert_eq!(res.success, true);
        assert_eq!(res.message, "Updated successfully".to_owned());
    }

    #[tokio::test]
    async fn test_add_balance_end_handler_failed_transaction() {
        let ts = get_epoch_ts();
        let token = "DUMMY_TOKEN";
        let user_id = 10;
        let amount = 10;
        let transaction_id = "6494fdba5155b267cb139995";
        let error_reason = "test error reason";
        let tracking_id = "test tracking id";
        let mut state = AppState::mock();
        state.get_mut_db().expect_execute_transaction().never();
        state
            .get_mut_utility()
            .expect_decode_token()
            .once()
            .with(eq(token))
            .returning(move |_| Ok(JwtClaims::new(user_id, None, false, ts as usize)));
        state
            .get_mut_validators()
            .expect_validate_add_bal_transaction()
            .once()
            .with(
                function(|db: &DbClient| true),
                function(|h: &Helpers| true),
                eq(user_id),
                function(move |body: &AddBalEndReq| {
                    body.amount == amount
                        && !body.is_successful
                        && body.transaction_id.to_hex().as_str() == transaction_id
                        && body.error_reason == Some(error_reason.to_owned())
                        && body.tracking_id == Some(tracking_id.to_owned())
                }),
            )
            .return_once(|_, _, _, _| Ok(()));
        state
            .get_mut_helpers()
            .mut_wallet_helpers()
            .expect_update_failed_transaction()
            .once()
            .with(
                function(|db: &DbClient| true),
                eq(user_id),
                eq(ObjectId::parse_str(transaction_id).unwrap()),
                eq(Some(error_reason.to_owned())),
                eq(Some(tracking_id.to_owned())),
            )
            .return_once(|_, _, _, _, _| Ok(()));
        let state = Arc::new(state);
        let app = build_app_routes(state);
        let path = "/api/v1/wallet/addBalanceEnd";
        let body = json!({
            "amount": amount,
            "isSuccessful": false,
            "transactionId": transaction_id,
            "errorReason": error_reason,
            "trackingId": tracking_id
        });
        let body = build_post_request(path, body.to_string().as_str(), Some(token));
        let res = oneshot_request::<GenericResponse>(app, body, Some(StatusCode::OK)).await;
        assert_eq!(res.success, true);
        assert_eq!(res.message, "Updated successfully".to_owned());
    }
}
