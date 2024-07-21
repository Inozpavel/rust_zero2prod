use crate::email_client::EmailClient;
use sqlx::PgPool;

#[derive(Debug)]
pub struct AppState {
    pub database: PgPool,
    pub email_client: EmailClient,
}
