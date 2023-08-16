use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initializes the logger with tracing_subscriber
pub fn initialize_logging() {
    // create default env filter
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or("treatviewers_backend=debug".into());

    // initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();
}
