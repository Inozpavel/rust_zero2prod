use std::sync::Arc;

use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::error::{ApplicationError, DomainError};

#[derive(Deserialize)]
pub struct ConfirmSubscriptionQuery {
    token: String,
}
#[tracing::instrument(skip_all)]
pub async fn confirm_subscription(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<ConfirmSubscriptionQuery>,
) -> Result<(), ApplicationError> {
    let subscriber_id = app_state
        .repository
        .get_subscriber_id_by_token(&query.token)
        .await?;

    if let Some(id) = subscriber_id {
        app_state
            .repository
            .update_subscriber_confirmation_status(&id)
            .await?;
    } else {
        return Err(DomainError::from("Token wasn't found").into());
    }

    Ok(())
}
