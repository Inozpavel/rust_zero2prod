use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::Once;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use zero2prod::app_config::{get_app_configuration, AppConfig};
use zero2prod::startup::build;

pub struct TestApp {
    pub base_address: String,
    pub pool: PgPool,
}

pub async fn spawn_app() -> Result<TestApp, anyhow::Error> {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let filter = EnvFilter::from(std::env::var("RUST_LOG").unwrap_or("INFO".into()));
        tracing_subscriber::fmt().with_env_filter(filter).init()
    });

    let mut configuration = build_test_app_config()?;
    configuration.database.database_name = Uuid::now_v7().to_string();

    let (listener, state) = build(&configuration).await?;

    let pool = state.database.clone();
    let given_port = listener.local_addr()?.port();

    configure_database(&configuration).await?;
    _ = tokio::task::spawn(zero2prod::run(state, configuration, listener));

    let base_address = format!("http://127.0.0.1:{}", given_port);
    let result = TestApp { base_address, pool };

    Ok(result)
}

async fn configure_database(config: &AppConfig) -> Result<PgPool, anyhow::Error> {
    let mut connection =
        PgConnection::connect_with(&config.database.without_database_name()).await?;

    let sql = format!(r#"CREATE DATABASE "{}";"#, config.database.database_name);
    connection.execute(sql.as_str()).await?;

    let pool_connection_options = config.database.with_database_name();
    let pool = PgPool::connect_with(pool_connection_options).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

fn build_test_app_config() -> Result<AppConfig, anyhow::Error> {
    let mut config = get_app_configuration()?;
    config.port = 0;
    config.database.database_name = Uuid::now_v7().to_string();

    Ok(config)
}
