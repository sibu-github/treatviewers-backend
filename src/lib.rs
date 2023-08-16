use std::{net::SocketAddr, sync::Arc};

mod config;
mod constants;
mod external_api;
mod handlers;
mod jobs;
mod models;
mod utils;
mod validators;

pub async fn start_web_server() {
    // read the port number from env variable
    let port = std::env::var("PORT").unwrap_or_default();
    let port = port.parse::<u16>().unwrap_or(3000);
    // build the socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    // initialize logging
    config::initialize_logging();
    // create AppState instance
    let state = config::AppState::new().await;
    let state = Arc::new(state);
    // build app routes
    let app = config::build_app_routes(state);
    tracing::debug!("Starting the app in: {addr}");
    // start serving the app in the socket address
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
