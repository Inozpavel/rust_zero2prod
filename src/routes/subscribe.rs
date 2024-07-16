use crate::app_state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}

pub async fn subscribe(
    app_state: State<Arc<AppState>>,
    Form(form): Form<SubscribeFormData>,
) -> Result<(), (StatusCode, String)> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, name, email, subscribed_at)
    VALUES ($1,$2,$3,$4)
    "#,
        Uuid::now_v7(),
        form.name,
        form.email,
        Utc::now(),
    )
    .execute(&app_state.database)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;
    info!("Form: {:?}", form);
    Ok(())
}
