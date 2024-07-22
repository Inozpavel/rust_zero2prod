use crate::app_config::AppConfig;
use crate::email_client::EmailClient;
use sqlx::PgPool;

#[derive(Debug)]
pub struct AppState {
    pub config: AppConfig,
    pub database: PgPool,
    pub email_client: EmailClient,
}
