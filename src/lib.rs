use crate::app_config::AppConfig;
use crate::app_state::AppState;
use crate::routes::subscribe::subscribe;
use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::DefaultOnResponse;
use tracing::info_span;

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
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|req: &Request<Body>| {
                    let req_id = req.headers()
                        .get("x-request-id")
                        .map(|v| v.to_str()
                            .unwrap_or("invalid UTF-8"))
                        .unwrap_or("None");
                    info_span!("http-request", method = ?req.method(), uri = ?req.uri(), ?req_id)
                })
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
        )
        .layer(axum::middleware::from_fn(override_code))
        .layer(tower_http::request_id::PropagateRequestIdLayer::x_request_id())
        .layer(tower_http::request_id::SetRequestIdLayer::x_request_id(
            tower_http::request_id::MakeRequestUuid,
        ))
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
