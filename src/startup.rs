use crate::app_config::AppConfig;
use crate::app_state::AppState;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::info;

pub async fn build(config: &AppConfig) -> Result<(TcpListener, AppState), anyhow::Error> {
    let db_pool_options = config.database.with_database_name();
    let db_pool = PgPoolOptions::new().connect_with(db_pool_options).await?;

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

    let state = AppState { database: db_pool };
    Ok((listener, state))
}
