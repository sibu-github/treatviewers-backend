use std::sync::Arc;

use axum::{body::Body, routing::post, Extension, Router};

use crate::config::AppState;

pub(crate) mod add_bal;
pub(crate) mod extension;

use add_bal::*;

#[cfg_attr(test, mockall_double::double)]
use self::extension::WalletExtension;

pub fn wallet_routes() -> Router<Arc<AppState>, Body> {
    let wallet_extension = Arc::new(WalletExtension::new());
    Router::new()
        .route("/addBalanceInit", post(add_bal_init_handler))
        .layer(Extension(wallet_extension))
}
