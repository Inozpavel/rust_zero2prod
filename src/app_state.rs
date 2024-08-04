use crate::app_config::AppConfig;
use crate::email_client::EmailClient;
use crate::infrastructure::sqlx_postgres_repository::SqlxPostgresRepository;
use std::fmt::Debug;

#[derive(Debug)]
pub struct AppState {
    pub config: AppConfig,
    pub repository: SqlxPostgresRepository,
    pub email_client: EmailClient,
}
