use crate::app_state::AppState;
use crate::domain::value_objects::{NewSubscriber, SubscriberEmail, SubscriberName};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}

#[tracing::instrument(skip(app_state))]
pub async fn subscribe(
    app_state: State<Arc<AppState>>,
    Form(form): Form<SubscribeFormData>,
) -> Result<(), (StatusCode, String)> {
    let subscriber = NewSubscriber {
        email: SubscriberEmail::parse(form.email).map_err(|e| (StatusCode::BAD_REQUEST, e))?,
        name: SubscriberName::parse(form.name).map_err(|e| (StatusCode::BAD_REQUEST, e))?,
    };

    insert_subscriber(&app_state.database, &subscriber)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;

    info!("New subscription!");

    Ok(())
}

#[derive(Debug)]
enum EmailStatus {
    Confirmed,
    Unconfirmed,
}

#[tracing::instrument(skip_all)]
async fn insert_subscriber(pool: &PgPool, subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, name, email, subscribed_at, status)
    VALUES ($1,$2,$3,$4,$5)
    "#,
        Uuid::now_v7(),
        subscriber.name.as_ref(),
        subscriber.email.as_ref(),
        Utc::now(),
        format!("{:?}", EmailStatus::Confirmed)
    )
    .execute(pool)
    .await
    .map_err(|e| {
        error!("Failed to execute query {:?}", e);
        e
    })?;

    Ok(())
}
