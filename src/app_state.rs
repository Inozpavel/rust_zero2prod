use sqlx::PgPool;

#[derive(Debug)]
pub struct AppState {
    pub database: PgPool,
}
