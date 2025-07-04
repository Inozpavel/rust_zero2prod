use crate::app_config::{AppConfig, DatabaseConfig};
use crate::app_state::AppState;
use crate::domain::value_objects::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::infrastructure::sqlx_postgres_repository::SqlxPostgresRepository;
use crate::middlewares::basic_auth::basic_auth;
use crate::routes::confirm_subscription::confirm_subscription;
use crate::routes::publish_newsletter::publish_newsletter;
use crate::routes::subscribe::subscribe;
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{error, info, info_span};

pub async fn build(config: AppConfig) -> Result<(TcpListener, AppState), anyhow::Error> {
    let db_pool = get_database_pool(&config.database).await?;

    sqlx::migrate!().run(&db_pool).await?;

    let address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(address).await?;

    let email_client = EmailClient::new(
        config.email_client.base_url.to_string(),
        config.email_client.authorization_token.to_string(),
        SubscriberEmail::parse(config.email_client.sender_email.to_string())
            .map_err(|e| anyhow!(e))?,
    );
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

    let repository = SqlxPostgresRepository::new(db_pool);

    let state = AppState {
        repository,
        email_client,
        config,
    };
    Ok((listener, state))
}

pub async fn get_database_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    let db_pool_options = config.with_database_name();
    let db_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_with(db_pool_options)
        .await?;

    Ok(db_pool)
}

pub async fn run_until_stopped(
    state: AppState,
    listener: TcpListener,
) -> Result<(), anyhow::Error> {
    let state = Arc::new(state);
    let router = Router::new()
        .route("/newsletter", post(publish_newsletter))
        .layer(axum::middleware::from_fn_with_state(state.clone(), basic_auth))
        .route("/health", get(|| async {}))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm_subscription))
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|req: &Request<Body>| {
                    let req_id = req
                        .headers()
                        .get("x-request-id")
                        .map(|v| v.to_str().unwrap_or("invalid UTF-8"))
                        .unwrap_or("None");

                    let span_name = format!("{} {}", req.method().as_str(), req.uri().path());
                    info_span!("http-request", method = ?req.method(), uri = ?req.uri().path(), ?req_id, otel.name=span_name)
                })
        )
        .layer(axum::middleware::from_fn(log_error_response))
        .layer(axum::middleware::from_fn(override_code))
        .layer(tower_http::request_id::PropagateRequestIdLayer::x_request_id())
        .layer(tower_http::request_id::SetRequestIdLayer::x_request_id(
            tower_http::request_id::MakeRequestUuid,
        ))
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
async fn log_error_response(req: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(req).await;
    let status = response.status_mut();

    if status.as_u16() >= 500 {
        error!("Processing error: {:?}", response.body())
    }

    response
}
