use axum::Router;
use axum::extract::{Path};
use axum::http::StatusCode;
use axum::routing::get;
use tokio::net::TcpListener;
use tracing::{info};
use tracing_subscriber::EnvFilter;

async fn greet_for_name(Path(q): Path<String>) -> (StatusCode, String) {
    (StatusCode::OK, format!("Hello, {}", q))
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::from(std::env::var("RUST_LOG").unwrap_or("INFO".into()));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("Listening http://{}", listener.local_addr()?);

    let router = Router::new()
        .route("/health", get(|| async {}))
        .route("/:name", get(greet_for_name))
        .into_make_service();
    axum::serve(listener, router).await?;

    Ok(())
}
