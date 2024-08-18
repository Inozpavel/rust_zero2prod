use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use derive_more::{Display, From};
use serde_json::json;
use std::borrow::Cow;
use std::fmt::Debug;
use tracing::error;

#[derive(Debug, From, Display)]
pub enum RepositoryError {
    Database(sqlx::Error),
    Domain(DomainError),
}

#[derive(Debug, From, Display)]
pub enum ApplicationError {
    RepositoryError(RepositoryError),
    InternalLogicDomainError(InternalLogicDomainError),
    InternalLogicError(InternalLogicError),
    AuthError(anyhow::Error),
    DomainError(DomainError),
}

#[derive(Debug, From, Display)]
pub struct DomainError(Cow<'static, str>);

#[derive(Debug, From, Display)]
pub struct InternalLogicDomainError(DomainError);

#[derive(Debug, From, Display)]
pub struct InternalLogicError(anyhow::Error);

impl From<String> for DomainError {
    fn from(value: String) -> Self {
        DomainError::from(Cow::Owned(value))
    }
}

impl From<&'static str> for DomainError {
    fn from(value: &'static str) -> Self {
        DomainError::from(Cow::Borrowed(value))
    }
}

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        match self {
            e @ ApplicationError::RepositoryError(..) => {
                error!("Processing error!\n{:#?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, to_json_error(e))
            }
            e @ ApplicationError::DomainError(..) => (StatusCode::BAD_REQUEST, to_json_error(e)),
            e @ ApplicationError::InternalLogicDomainError(..) => {
                (StatusCode::INTERNAL_SERVER_ERROR, to_json_error(e))
            }
            e @ ApplicationError::InternalLogicError(..) => {
                (StatusCode::INTERNAL_SERVER_ERROR, to_json_error(e))
            }
            e @ ApplicationError::AuthError(..) => (StatusCode::UNAUTHORIZED, to_json_error(e)),
        }
        .into_response()
    }
}

fn to_json_error<T: Display + Debug>(error: T) -> Response {
    let message = format!("{}", error);
    let details = format!("{:?}", error);
    let json = json!({
        "message": message,
        "details": details
    });
    Json(json).into_response()
}
