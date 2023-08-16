use std::sync::Arc;

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::{Request, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
    TypedHeader,
};

use crate::config::AppError;

use super::AppState;

pub async fn auth_middleware<B>(
    uri: Uri,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    State(state): State<Arc<AppState>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    tracing::debug!("Running auth_middleware: path = {}", uri.path());
    if state.utility().is_admin_only_path(&uri) || !state.utility().is_unprotected_path(&uri) {
        let Some(bearer) = bearer else {
            return AppError::Auth("missing token".into()).into_response();
        };
        match state.utility().decode_token(bearer.token()) {
            Ok(claims) => {
                if state.utility().is_admin_only_path(&uri) && !claims.is_admin {
                    let err = "Unauthorized for ADMIN ONLY path";
                    return AppError::Auth(err.into()).into_response();
                    // TODO: implement extra check to validate admin user from database
                }
                request.extensions_mut().insert(claims);
            }
            Err(e) => {
                return e.into_response();
            }
        }
    }
    let response = next.run(request).await;
    response
}
