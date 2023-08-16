use std::sync::Arc;

use axum::{body::Body, middleware, routing::get, Router};

use crate::handlers::*;

use super::{app_state::AppState, auth_middleware};

pub fn api_routes(state: Arc<AppState>) -> Router<Arc<AppState>, Body> {
    Router::new()
        .route("/ping", get(ping_handler))
        .route("/tempApiGetToken", get(temp_api_get_token))
        .route("/tempApiGetOtp", get(temp_api_get_otp))
        .nest("/wallet", wallet_routes())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}
