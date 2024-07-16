use anyhow::Context;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub host: String,
    pub database: DatabaseConfig,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseConfig {
    pub fn build_postgres_connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

pub fn get_app_configuration() -> Result<AppConfig, anyhow::Error> {
    let config = config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()
        .context("Build app config")?;

    let app_config = config
        .try_deserialize::<AppConfig>()
        .context("Deserialize app config")?;

    info!("Config: {:?} ", app_config);
    Ok(app_config)
}
