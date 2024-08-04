use std::sync::Arc;

use crate::app_state::AppState;
use crate::error::ApplicationError;
use axum::extract::{Query, State};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfirmSubscriptionQuery {
    token: String,
}
#[tracing::instrument(skip_all)]
pub async fn confirm_subscription(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<ConfirmSubscriptionQuery>,
) -> Result<(), ApplicationError<'static>> {
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
        return Err(ApplicationError::DomainError("Token wasn't found".into()));
    }

    Ok(())
}
