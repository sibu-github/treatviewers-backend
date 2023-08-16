use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use serde::Deserialize;
use utoipa::IntoParams;
use validator::Validate;

use crate::{
    config::{AppError, AppState},
    constants::*,
    models::*,
    validators::validate_phonenumber,
};

/// Ping endpoint
///
/// Ping the server to get a static response
#[utoipa::path(
    get,
    path = "/api/v1/ping",
    responses(
        (status = 200, description = "Get success response from server", body = GenericResponse)
    ),
    tag = "Debugging API"
)]
pub async fn ping_handler() -> Json<GenericResponse> {
    GenericResponse::json_response(true, "Server running successfully!")
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    user_id: u32,
    name: Option<String>,
    is_admin: Option<bool>,
}

/// Temporary API to get token
///
/// Returns a JWT token for an user
#[utoipa::path(
    get,
    path = "/api/v1/tempApiGetToken",
    params(Params),
    responses(
        (status = 200, description = "Get JWT token for an user", body = GenericResponse)
    ),
    tag = "Debugging API"
)]
pub async fn temp_api_get_token(
    State(state): State<Arc<AppState>>,
    Query(params): Query<Params>,
) -> Result<Json<GenericResponse>, AppError> {
    let token = match params.is_admin {
        Some(true) => state
            .utility()
            .generate_token_admin(params.user_id, params.name)?,
        _ => state
            .utility()
            .generate_token(params.user_id, params.name)?,
    };

    Ok(GenericResponse::json_response(true, &token))
}

#[derive(Debug, Deserialize, Validate, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct OtpParams {
    #[validate(custom(function = "validate_phonenumber"))]
    phone: String,
    admin_otp: Option<bool>,
}

/// Temporary API to get OTP
///
/// Returns the generated OTP for an user
#[utoipa::path(
    get,
    path = "/api/v1/tempApiGetOtp",
    params(OtpParams),
    responses(
        (status = 200, description = "Get OTP for an user", body = GenericResponse)
    ),
    tag = "Debugging API"
)]
pub async fn temp_api_get_otp(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OtpParams>,
) -> Result<Json<GenericResponse>, AppError> {
    params
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let filter = Some(doc! {"phone": &params.phone});
    let user_id = if params.admin_otp == Some(true) {
        let user = state
            .db()
            .find_one::<AdminUser>(DB_NAME, COLL_ADMIN_USERS, filter, None)
            .await?
            .ok_or(AppError::NotFound("User not found".into()))?;
        user.id
    } else {
        let user = state
            .db()
            .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
            .await?
            .ok_or(AppError::NotFound("User not found".into()))?;
        user.id
    };
    let ts = state.utility().get_epoch_ts() as i64;
    let filter = doc! {"userId": user_id, "validTill": {"$gte": ts}, "isUsed": false};
    let otp = state
        .db()
        .find_one::<Otp>(DB_NAME, COLL_OTP, Some(filter), None)
        .await?
        .ok_or(anyhow::anyhow!("Otp not found"))?;
    Ok(GenericResponse::json_response(true, &otp.otp))
}
