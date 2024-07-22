use crate::app_state::AppState;
use crate::domain::entities::subscriber::Subscriber;
use crate::domain::value_objects::{
    ConfirmationStatus, SubscriberEmail, SubscriberId, SubscriberName,
};
use crate::email_client::EmailClient;
use crate::error::ApplicationError;
use crate::error::RepositoryError::DomainError;
use axum::extract::State;
use axum::Form;
use chrono::Utc;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use sqlx::{Executor, Postgres};
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
) -> Result<(), ApplicationError<'static>> {
    let subscriber = Subscriber {
        id: SubscriberId::new(),
        email: SubscriberEmail::parse(form.email).map_err(DomainError)?,
        name: SubscriberName::parse(form.name).map_err(DomainError)?,
    };

    let token = generate_subscription_token();

    let transaction = &mut app_state.database.begin().await?;
    insert_subscriber(&mut **transaction, &subscriber).await?;

    store_token(&mut **transaction, &subscriber, &token).await?;

    send_confirmation_email(
        &subscriber,
        &app_state.email_client,
        &token,
        &app_state.config.base_url,
    )
    .await
    .map_err(|e| DomainError(format!("{}", e)))?;

    info!("New subscription!");

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn insert_subscriber(
    executor: impl Executor<'_, Database = Postgres>,
    subscriber: &Subscriber,
) -> Result<(), sqlx::Error> {
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
    .execute(executor)
    .await
    .map_err(|e| {
        error!("Failed to execute query {:?}", e);
        e
    })?;

    Ok(())
}

async fn store_token(
    executor: impl Executor<'_, Database = Postgres>,
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
    .execute(executor)
    .await?;

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn send_confirmation_email(
    subscriber: &Subscriber,
    email_client: &EmailClient,
    confirmation_token: &str,
    base_url: &str,
) -> Result<(), anyhow::Error> {
    let subject = "Welcome!";
    let confirmation_link = format!(
        "{}/subscriptions/confirm?token={}",
        base_url, confirmation_token
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
