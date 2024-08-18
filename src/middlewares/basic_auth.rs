use crate::app_state::AppState;
use crate::domain::value_objects::PasswordHash;
use crate::error::ApplicationError;
use anyhow::{anyhow, bail, Context};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;

struct BasicAuthCredentials {
    username: String,
    password: String,
}
pub async fn basic_auth(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, ApplicationError> {
    let credentials =
        extract_credentials(&req).map_err(|e| ApplicationError::AuthError(e.into()))?;

    let password_hash = PasswordHash::new_from_password(&credentials.password);
    let user_exists = state
        .repository
        .user_by_credentials_exists(&credentials.username, &password_hash)
        .await
        .map_err(ApplicationError::RepositoryError)?;

    if !user_exists {
        return Err(ApplicationError::AuthError(anyhow!("User wasn't found")));
    }

    let response = next.run(req).await;

    Ok(response)
}

fn extract_credentials(req: &Request) -> Result<BasicAuthCredentials, anyhow::Error> {
    let auth_header_value = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| anyhow!("Authorization header value is missing"))?
        .to_str()
        .context("Authorization header value is not valid UTF-8")?;

    let mut splitted = auth_header_value.split(' ');
    let (auth_type, value) = (splitted.next(), splitted.next());

    match auth_type {
        None => bail!("Authorization header value auth scheme type is missing"),
        Some("Basic") => {}
        Some(_) => bail!("Authorization scheme is not Basic"),
    }

    let Some(credentials) = value else {
        bail!("Authorization header value credentials");
    };

    let decoded_bytes =
        base64_url::decode(credentials).context("Failed to to decode base64 credentials")?;

    let decoded_credentials = std::str::from_utf8(&decoded_bytes)
        .context("Decoded credentials string isn't valid UTF-8 string")?;

    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow!("Username must be provided"))?
        .to_string();

    let password = credentials
        .next()
        .ok_or_else(|| anyhow!("Password must be provided"))?
        .to_string();

    Ok(BasicAuthCredentials { username, password })
}
