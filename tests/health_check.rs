use std::time::Duration;
use const_format::concatcp;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format;
use zero2prod::config::AppConfig;

#[tokio::test]
async fn health_check_works() -> Result<(), anyhow::Error> {
    let base_address = spawn_app().await?;

    let url = format!("{}/health", base_address);
    let response = reqwest::get(url).await?;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    Ok(())
}

async fn spawn_app() -> Result<String, anyhow::Error> {
    let filter = EnvFilter::from(std::env::var("RUST_LOG").unwrap_or("INFO".into()));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = AppConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
    };

    let address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(address).await?;
    let given_port = listener.local_addr()?.port();

    info!("Listening http://{}", listener.local_addr()?);
    tokio::task::spawn(zero2prod::run(listener));

    let base_address = format!("http://127.0.0.1:{}", given_port);
    Ok(base_address)
}
