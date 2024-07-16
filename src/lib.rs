use crate::app_config::AppConfig;
use crate::app_state::AppState;
use crate::routes::subscribe::subscribe;
use axum::extract::{Request};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Router};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;

pub mod app_config;
pub mod app_state;
pub mod routes;

pub async fn run(config: AppConfig, listener: TcpListener) -> Result<(), anyhow::Error> {
    let pool = PgPool::connect(&config.database.build_postgres_connection_string()).await?;
    let state = AppState { database: pool };
    let state = Arc::new(state);

    let router = Router::new()
        .route("/health", get(|| async {}))
        .route("/subscribe", post(subscribe))
        .layer(axum::middleware::from_fn(override_code))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Route wasn't found") })
        .with_state(state)
        .into_make_service();
    axum::serve(listener, router).await?;
    Ok(())
}

async fn override_code(req: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(req).await;

    let status = response.status_mut();

    if *status == StatusCode::UNPROCESSABLE_ENTITY {
        *status = StatusCode::BAD_REQUEST;
    }

    response
}
