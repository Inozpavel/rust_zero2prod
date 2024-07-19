use crate::app_config::AppConfig;
use crate::app_state::AppState;
use crate::routes::subscribe::subscribe;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;

pub mod app_config;
pub mod app_state;
pub mod routes;

pub async fn run(
    state: AppState,
    _config: AppConfig,
    listener: TcpListener,
) -> Result<(), anyhow::Error> {
    let router = Router::new()
        .route("/health", get(|| async {}))
        .route("/subscribe", post(subscribe))
        .layer(axum::middleware::from_fn(override_code))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .fallback(|| async { (StatusCode::NOT_FOUND, "Route wasn't found") })
        .with_state(Arc::new(state))
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
