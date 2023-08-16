use std::sync::Arc;

use axum::{
    async_trait,
    extract::FromRequest,
    http::Request,
    response::{IntoResponse, Response},
    Json, RequestExt,
};
use validator::Validate;

use crate::validators::ValidateExtra;

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
        data.validate_extra(state.clone()).await.map_err(|err| {
            tracing::debug!("Error validating json body with extra function: {:?}", err);
            err.into_response()
        })?;

        // return the validated body
        Ok(Self(data))
    }
}
