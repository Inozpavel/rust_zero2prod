use axum::extract::Query;
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfirmSubscriptionQuery {
    token: String,
}
pub async fn confirm_subscription(
    Query(query): Query<ConfirmSubscriptionQuery>,
) -> Result<(), (StatusCode, String)> {
    Ok(())
}
