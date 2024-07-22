#![deny(unused_imports)]

use tracing_subscriber::EnvFilter;
use zero2prod::app_config::get_app_configuration;
use zero2prod::startup::{build, run_until_stopped};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let log_level = std::env::var("RUST_LOG").unwrap_or("trace".into());
    let filter = EnvFilter::builder().parse(log_level)?;
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = get_app_configuration()?;

    let (listener, state) = build(config).await?;

    run_until_stopped(state, listener).await
}
