use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;
use zero2prod::config::AppConfig;
use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::from(std::env::var("RUST_LOG").unwrap_or("INFO".into()));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = AppConfig {
        port: 8000,
        host: "127.0.0.1".to_string(),
    };

    let address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(address).await?;
    info!("Listening http://{}", listener.local_addr()?);

    run(listener).await
}
