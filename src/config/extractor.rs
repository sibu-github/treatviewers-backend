use std::sync::Arc;

use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, Query},
    http::{request::Parts, Request},
    response::{IntoResponse, Response},
    Extension, Json, RequestExt, RequestPartsExt,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::{models::JwtClaims, validators::ValidateExtra};

use super::{error_handler::AppError, AppState};

pub struct ValidatedBody<T>(pub T);

#[async_trait]
impl<B, T> FromRequest<Arc<AppState>, B> for ValidatedBody<T>
where
    B: Send + 'static,
    T: Validate + ValidateExtra + Sync + Send + 'static,
    Json<T>: FromRequest<(), B>,
{
    type Rejection = Response;

    async fn from_request(req: Request<B>, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let claims = req.extensions().get::<JwtClaims>();
        let user_id = claims.and_then(|user| Some(user.id));

        // extract the JSON body
        let Json(data) = req.extract::<Json<T>, _>().await.map_err(|err| {
            let msg = format!("Error extracting the JSON body");
            tracing::debug!(msg);
            err.into_response()
        })?;
        // validate json body
        data.validate().map_err(|err| {
            let msg = format!("Error validating json body: {err}");
            tracing::debug!(msg);
            AppError::BadRequest(msg).into_response()
        })?;
        // validate extra with AppState
        data.validate_extra(state.clone(), user_id)
            .await
            .map_err(|err| {
                tracing::debug!("Error validating json body with extra function: {:?}", err);
                err.into_response()
            })?;

        // return the validated body
        Ok(Self(data))
    }
}

pub struct ValidatedParams<T>(pub T);

#[async_trait]
impl<T> FromRequestParts<Arc<AppState>> for ValidatedParams<T>
where
    T: Validate + ValidateExtra + Sync + Send + 'static,
    T: DeserializeOwned,
    Query<T>: FromRequestParts<Arc<AppState>>,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let params = parts
            .extract::<Query<T>>()
            .await
            .map(|Query(params)| params)
            .map_err(|err| err.into_response())?;
        let claims = parts.extensions.get::<JwtClaims>();
        let user_id = claims.and_then(|user| Some(user.id));
        params.validate().map_err(|err| {
            let msg = format!("Error validating json query params: {err}");
            tracing::debug!(msg);
            AppError::BadRequest(msg).into_response()
        })?;
        params
            .validate_extra(state.clone(), user_id)
            .await
            .map_err(|err| {
                tracing::debug!(
                    "Error validating json query params with extra function: {:?}",
                    err
                );
                err.into_response()
            })?;
        Ok(Self(params))
    }
}
