mod default_route;
mod global_404;

#[cfg(test)]
mod extra_test_routes;

pub use default_route::default_route_handler;
pub use global_404::global_404_handler;

#[cfg(test)]
pub use extra_test_routes::*;
