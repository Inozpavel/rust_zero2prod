use anyhow::Context;
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
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

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseConfig {
    pub fn without_database_name(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .username(&self.username)
            .password(&self.password)
            .host(&self.host)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_database_name(&self) -> PgConnectOptions {
        self.without_database_name().database(&self.database_name)
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
