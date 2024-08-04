use reqwest::Response;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::collections::HashMap;
use std::sync::Once;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use zero2prod::app_config::{get_app_configuration, AppConfig};
use zero2prod::startup::{build, get_database_pool};

pub struct TestApp {
    pub base_address: String,
    pub pool: PgPool,
    client: reqwest::Client,
}

impl TestApp {
    pub async fn post_subscriptions(
        &self,
        map: &HashMap<&str, &str>,
    ) -> Result<Response, reqwest::Error> {
        let url = format!("{}/subscriptions", self.base_address);
        let response = self.client.post(&url).form(&map).send().await?;

        Ok(response)
    }
}

pub async fn spawn_app() -> Result<TestApp, anyhow::Error> {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let filter = EnvFilter::from(std::env::var("RUST_LOG").unwrap_or("INFO".into()));
        tracing_subscriber::fmt().with_env_filter(filter).init()
    });

    let configuration = build_test_app_config()?;
    configure_database(&configuration).await?;

    let (listener, state) = build(configuration).await?;

    let given_port = listener.local_addr()?.port();

    let pool = state.repository.inner().clone();
    _ = tokio::task::spawn(zero2prod::startup::run_until_stopped(state, listener));

    let base_address = format!("http://127.0.0.1:{}", given_port);
    let result = TestApp {
        base_address,
        pool,
        client: reqwest::Client::new(),
    };

    Ok(result)
}

async fn configure_database(config: &AppConfig) -> Result<PgPool, anyhow::Error> {
    let mut connection =
        PgConnection::connect_with(&config.database.without_database_name()).await?;

    let sql = format!(r#"CREATE DATABASE "{}";"#, config.database.database_name);
    connection.execute(sql.as_str()).await?;

    let pool = get_database_pool(&config.database).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

fn build_test_app_config() -> Result<AppConfig, anyhow::Error> {
    let mut config = get_app_configuration()?;
    config.port = 0;
    config.database.database_name = Uuid::now_v7().to_string();

    Ok(config)
}
