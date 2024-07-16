use sqlx::PgPool;

pub struct AppState {
    pub database: PgPool,
}
