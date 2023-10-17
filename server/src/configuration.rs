use anyhow::Context;
use base64::engine::general_purpose;
use base64::Engine;
use deadpool_redis::redis::IntoConnectionInfo;
use jsonwebtoken::{DecodingKey, EncodingKey};
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;
use std::convert::{TryFrom, TryInto};

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub auth: AuthSettings,
    pub redis: RedisSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct RedisSettings {
    pub username: Option<String>,
    pub password: Option<Secret<String>>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database: Option<i64>,
}

impl RedisSettings {
    pub fn from_config(&self) -> deadpool_redis::Config {
        let db_url = format!("redis://{}:{}", self.host, self.port);
        let mut conn_info = db_url.clone().into_connection_info().unwrap();
        // conn_info.redis.password = .map(|pw| pw.expose_secret().to_string());
        // conn_info.redis.username = self.username.map(|uname| uname.to_string());
        if let Some(database) = self.database {
            conn_info.redis.db = database;
        }
        deadpool_redis::Config::from_connection_info(conn_info)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct AuthSettings {
    //TODO: add refresh keys
    pub eddsa_private_key_pem: Secret<String>,
    pub eddsa_public_key_pem: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub access_toked_duration: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub refresh_toked_duration: u64,
}

impl AuthSettings {
    pub fn encoding_key(&self) -> Result<EncodingKey, anyhow::Error> {
        let bytes = general_purpose::STANDARD.decode(self.eddsa_private_key_pem.expose_secret())?;
        EncodingKey::from_ed_pem(&bytes).context("Invalid privet key pem.")
    }

    pub fn decoding_key(&self) -> Result<DecodingKey, anyhow::Error> {
        let bytes = general_purpose::STANDARD.decode(&self.eddsa_public_key_pem)?;
        DecodingKey::from_ed_pem(&bytes).context("Invalid public key pem.")
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
