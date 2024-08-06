use crate::app_state::AppState;
use crate::domain::entities::subscriber::Subscriber;
use crate::domain::value_objects::{SubscriberEmail, SubscriberId, SubscriberName};
use crate::email_client::EmailClient;
use crate::error::{ApplicationError, DomainError, RepositoryError};
use axum::extract::State;
use axum::Form;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}

#[tracing::instrument(skip(app_state))]
pub async fn subscribe(
    State(app_state): State<Arc<AppState>>,
    Form(form): Form<SubscribeFormData>,
) -> Result<(), ApplicationError> {
    let subscriber = Subscriber {
        id: SubscriberId::new(),
        email: SubscriberEmail::parse(form.email)?,
        name: SubscriberName::parse(form.name)?,
    };
    let token = generate_subscription_token();

    {
        let mut transaction = app_state.repository.begin_transaction().await?;
        app_state
            .repository
            .insert_subscriber_tx(&mut transaction, &subscriber)
            .await?;

        app_state
            .repository
            .store_token_tx(&mut transaction, &subscriber, &token)
            .await?;

        transaction.commit().await.map_err(RepositoryError::from)?;
    }

    send_confirmation_email(
        &subscriber,
        &app_state.email_client,
        &token,
        &app_state.config.base_url,
    )
    .await
    .map_err(|e| DomainError::from(format!("{}", e)))?;

    info!("New subscription!");

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
