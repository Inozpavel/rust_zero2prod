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
    DatabaseError(sqlx::Error),
    DomainError(String),
}

#[derive(Debug, From, Display)]
pub enum ApplicationError<'a> {
    RepositoryError(sqlx::Error),
    DomainError(Cow<'a, str>),
}

impl<'a> IntoResponse for ApplicationError<'a> {
    fn into_response(self) -> Response {
        match self {
            e @ ApplicationError::RepositoryError(..) => {
                error!("Processing error!\n{:#?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, to_json_error(e))
            }
            e @ ApplicationError::DomainError(..) => (StatusCode::BAD_REQUEST, to_json_error(e)),
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

impl<'a> From<RepositoryError> for ApplicationError<'a> {
    fn from(value: RepositoryError) -> Self {
        match value {
            RepositoryError::DatabaseError(database_error) => {
                ApplicationError::RepositoryError(database_error)
            }
            RepositoryError::DomainError(domain_error) => {
                ApplicationError::DomainError(domain_error.into())
            }
        }
    }
}
