use axum::extract::Path;
use axum::http::StatusCode;
use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;

pub mod config;

pub async fn run(listener: TcpListener) -> Result<(), anyhow::Error> {
    let router = Router::new()
        .route("/health", get(|| async {}))
        .route("/:name", get(greet_for_name))
        .into_make_service();
    axum::serve(listener, router).await?;
    Ok(())
}

async fn greet_for_name(Path(q): Path<String>) -> (StatusCode, String) {
    (StatusCode::OK, format!("Hello, {}", q))
}
