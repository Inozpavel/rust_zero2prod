use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::str::FromStr;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;
use zero2prod::app_config::get_app_configuration;
use zero2prod::app_state::AppState;
use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let log_level = std::env::var("RUST_LOG").unwrap_or("trace".into());
    let filter = EnvFilter::builder().parse(log_level)?;
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = get_app_configuration()?;

    let address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(address).await?;

    let addr = listener.local_addr()?;
    if addr.ip().is_unspecified() {
        info!(
            "Listening http://{}. For debug use: http://127.0.0.1:{}",
            addr,
            addr.port()
        );
    } else {
        info!("Listening http://{}", listener.local_addr()?);
    }

    let connect_options = config.database.with_database_name();
    let pool = PgPoolOptions::new().connect_with(connect_options).await?;

    let state = AppState { database: pool };

    run(state, config, listener).await
}
