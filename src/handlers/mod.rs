pub(crate) mod default_route;
pub(crate) mod global_404;
pub(crate) mod ping;
pub(crate) mod wallet;

#[cfg(test)]
mod extra_test_routes;

pub use default_route::default_route_handler;
pub use global_404::global_404_handler;
pub use ping::*;
pub use wallet::wallet_routes;

#[cfg(test)]
pub use extra_test_routes::*;
