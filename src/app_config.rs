use anyhow::Context;
use serde::Deserialize;
use strum_macros::{Display, EnumString};
use tracing::info;

#[derive(Debug, Default, Eq, PartialEq, EnumString, Display)]
pub enum AppEnvironment {
    #[default]
    Local,
    Development,
    Staging,
    Production,
    Testing,
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    #[serde(skip)]
    pub environment: AppEnvironment,

    pub port: u16,
    pub host: String,
    pub database: DatabaseConfig,
}

impl AppConfig {
    pub fn database_connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database_name
        )
    }
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
    pub fn database_connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn database_connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub fn get_app_configuration() -> Result<AppConfig, anyhow::Error> {
    let environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or(format!("{}", AppEnvironment::Local))
        .parse::<AppEnvironment>()
        .context("App environment detection")?;

    let config = config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .add_source(
            config::File::with_name(&format!("configuration.{}", environment)).required(false),
        )
        .add_source(config::Environment::with_prefix("APP").separator("__"))
        .build()
        .context("Build app config")?;

    let mut app_config = config
        .try_deserialize::<AppConfig>()
        .context("Deserialize app config")?;

    app_config.environment = environment;

    info!("Config: {:?} ", app_config);

    Ok(app_config)
}
