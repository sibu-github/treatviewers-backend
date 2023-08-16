use std::time::Duration;

use tokio::time::sleep;

use crate::constants::REQUEST_TIMEOUT_SECS;

/// invokes sleep method to cause request timeout
/// to be used for testing timeout layer
pub async fn timeout_route_handler() -> String {
    sleep(Duration::from_secs(REQUEST_TIMEOUT_SECS + 1)).await;
    "Response from timeout_route_handler".to_owned()
}

/// invokes panic! to cause a panic
/// to be used for testing catch_panic layer
pub async fn catch_panic_handler() -> String {
    panic!("Invoking panic here");
}
