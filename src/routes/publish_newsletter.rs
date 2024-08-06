use crate::app_state::AppState;
use crate::error::{ApplicationError, InternalLogicError};
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct BodyData {
    title: String,
    content: BodyContent,
}

#[derive(Deserialize)]
pub struct BodyContent {
    text_content: String,
    html_content: String,
}
#[tracing::instrument(skip_all)]
pub async fn publish_newsletter(
    app_state: State<Arc<AppState>>,
    Json(body_data): Json<BodyData>,
) -> Result<(), ApplicationError> {
    let emails = app_state.repository.get_confirmed_emails().await?;

    for email in emails {
        app_state
            .email_client
            .send(
                &email,
                &body_data.title,
                &body_data.content.html_content,
                &body_data.content.text_content,
            )
            .await
            .map_err(InternalLogicError::from)?;
    }
    Ok(())
}
