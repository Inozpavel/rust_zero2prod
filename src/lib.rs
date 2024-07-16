use axum::extract::{Path, Request};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use tokio::net::TcpListener;
use tracing::info;

pub mod config;

async fn override_code(req: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(req).await;

    let status = response.status_mut();

    if *status == StatusCode::UNPROCESSABLE_ENTITY {
        *status = StatusCode::BAD_REQUEST;
    }

    response
}

pub async fn run(listener: TcpListener) -> Result<(), anyhow::Error> {
    let router = Router::new()
        .route("/health", get(|| async {}))
        .route("/subscribe", post(subscribe))
        .route("/hello/:name", get(greet_for_name))
        .layer(axum::middleware::from_fn(override_code))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Route wasn't found") })
        .into_make_service();
    axum::serve(listener, router).await?;
    Ok(())
}

async fn greet_for_name(Path(q): Path<String>) -> (StatusCode, String) {
    (StatusCode::OK, format!("Hello, {}", q))
}

#[derive(Deserialize, Debug)]
struct SubscribeFormData {
    name: String,
    email: String,
}
async fn subscribe(Form(form): Form<SubscribeFormData>) -> StatusCode {
    info!("Form: {:?}", form);
    StatusCode::OK
}
