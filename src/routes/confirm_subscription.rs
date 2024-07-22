use std::sync::Arc;

use axum::extract::{Query, State};
use serde::Deserialize;
use sqlx::PgPool;

use crate::app_state::AppState;
use crate::domain::value_objects::{ConfirmationStatus, SubscriberId};
use crate::error::{ApplicationError, RepositoryError};

#[derive(Deserialize)]
pub struct ConfirmSubscriptionQuery {
    token: String,
}
#[tracing::instrument(skip_all)]
pub async fn confirm_subscription(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<ConfirmSubscriptionQuery>,
) -> Result<(), ApplicationError<'static>> {
    let subscriber_id = get_subscriber_id_by_token(&app_state.database, &query.token).await?;

    if let Some(id) = subscriber_id {
        update_subscriber_confirmation_status(&app_state.database, &id).await?;
    } else {
        return Err(ApplicationError::DomainError("Token wasn't found".into()));
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn get_subscriber_id_by_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<SubscriberId>, RepositoryError> {
    let id = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens WHERE subscription_token=$1",
        token
    )
    .fetch_optional(pool)
    .await?
    .map(|x| x.subscriber_id);

    let id = id
        .map(|id| SubscriberId::parse(&id).map_err(RepositoryError::DomainError))
        .transpose()?;

    Ok(id)
}

#[tracing::instrument(skip_all)]
async fn update_subscriber_confirmation_status(
    pool: &PgPool,
    subscriber_id: &SubscriberId,
) -> Result<(), RepositoryError> {
    sqlx::query!(
        "UPDATE subscriptions SET status=$1 WHERE id=$2",
        ConfirmationStatus::Confirmed.as_ref(),
        subscriber_id.as_ref()
    )
    .execute(pool)
    .await?;

    Ok(())
}
