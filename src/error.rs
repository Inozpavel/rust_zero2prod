use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error("Domain error: {0}")]
    DomainError(String),
}

#[derive(Debug, Error)]
pub enum ApplicationError<'a> {
    #[error(transparent)]
    RepositoryError(#[from] sqlx::Error),
    #[error("Domain error: {0}")]
    DomainError(Cow<'a, str>),
}

impl<'a> IntoResponse for ApplicationError<'a> {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApplicationError::RepositoryError(repository_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{:?}", repository_error),
            ),
            ApplicationError::DomainError(domain_error) => {
                (StatusCode::BAD_REQUEST, format!("{}", domain_error))
            }
        }
        .into_response()
    }
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
