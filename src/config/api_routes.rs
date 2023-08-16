use std::sync::Arc;

use axum::{body::Body, middleware, routing::get, Router};

use crate::handlers::*;

use super::{app_state::AppState, auth_middleware};

pub fn api_routes(state: Arc<AppState>) -> Router<Arc<AppState>, Body> {
    Router::new()
        .route("/ping", get(|| async { "ping handler" }))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}
