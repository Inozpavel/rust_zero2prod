use crate::app_state::AppState;
use crate::domain::entities::subscriber::Subscriber;
use crate::domain::value_objects::{
    ConfirmationStatus, SubscriberEmail, SubscriberId, SubscriberName,
};
use crate::email_client::EmailClient;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;
use chrono::Utc;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};

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
    let subscriber = Subscriber {
        id: SubscriberId::new(),
        email: SubscriberEmail::parse(form.email).map_err(|e| (StatusCode::BAD_REQUEST, e))?,
        name: SubscriberName::parse(form.name).map_err(|e| (StatusCode::BAD_REQUEST, e))?,
    };

    let token = generate_subscription_token();
    insert_subscriber(&app_state.database, &subscriber)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;

    store_token(&app_state.database, &subscriber, &token)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;

    send_confirmation_email(&subscriber, &app_state.email_client, &token)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;

    info!("New subscription!");

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn insert_subscriber(pool: &PgPool, subscriber: &Subscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, name, email, subscribed_at, status)
    VALUES ($1,$2,$3,$4,$5)
    "#,
        subscriber.id.as_ref(),
        subscriber.name.as_ref(),
        subscriber.email.as_ref(),
        Utc::now(),
        ConfirmationStatus::PendingConfirmation.as_ref()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        error!("Failed to execute query {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn send_confirmation_email(
    subscriber: &Subscriber,
    email_client: &EmailClient,
    confirmation_token: &str,
) -> Result<(), anyhow::Error> {
    let subject = "Welcome!";
    let confirmation_link = format!(
        "https://my-api.com/subscriptions/confirm?token={}",
        confirmation_token
    );

    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription",
        confirmation_link
    );

    let html_body = format!(
        "Welcome to our newsletter!<br />\
Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send(&subscriber.email, subject, &html_body, &plain_body)
        .await?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

async fn store_token(
    pool: &PgPool,
    subscriber: &Subscriber,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscription_tokens (subscriber_id, subscription_token)
    VALUES ($1, $2)
    "#,
        &subscriber.id.as_ref().to_string(),
        token
    )
    .execute(pool)
    .await?;

    Ok(())
}
