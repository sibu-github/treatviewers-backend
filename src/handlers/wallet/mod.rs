use std::sync::Arc;

use axum::{body::Body, routing::post, Router};

use crate::config::AppState;

pub(crate) mod add_bal;

use add_bal::*;

pub fn wallet_routes() -> Router<Arc<AppState>, Body> {
    Router::new()
        .route("/addBalanceInit", post(add_bal_init_handler))
        .route("/addBalanceEnd", post(add_bal_end_handler))
}
