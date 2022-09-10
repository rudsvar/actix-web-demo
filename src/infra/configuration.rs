//! Structs and functions for reading application configuration from a file.

use serde::Deserialize;

use super::error::AppError;

/// Application settings.
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    /// Application settings.
    pub application: ApplicationSettings,
    /// Server settings.
    pub server: ServerSettings,
    /// Security settings.
    pub security: SecuritySettings,
    /// Database settings.
    pub database: DatabaseSettings,
    /// Logging settings.
    pub logging: LoggingSettings,
}

/// Application settings.
#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationSettings {
    /// Application name
    pub name: String,
}

/// Server settings.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerSettings {
    /// Server address.
    pub address: String,
    /// Server http port.
    pub http_port: u16,
    /// Server https port.
    pub https_port: u16,
    /// Server http port.
    pub grpc_address: String,
    /// Server https port.
    pub grpc_port: u16,
}

/// Security settings.
#[derive(Clone, Debug, Deserialize)]
pub struct SecuritySettings {
    /// SSL certificate.
    pub tls_certificate: String,
    /// SSL private key.
    pub tls_private_key: String,
    /// JWT private key.
    pub jwt_private_key: String,
    /// JWT public key.
    pub jwt_public_key: String,
    /// JWT public key.
    pub jwt_minutes_to_live: i64,
    /// Signing private key.
    pub signing_private_key: String,
    /// Signing public key.
    pub signing_public_key: String,
}

/// Database settings.
#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseSettings {
    /// The database username.
    pub username: String,
    /// The database password.
    pub password: String,
    /// The database port.
    pub port: u16,
    /// The database host.
    pub host: String,
    /// The database name.
    pub database_name: String,
}

/// Logging formats.
#[derive(Copy, Clone, Debug, Deserialize)]
pub enum LogFormat {
    /// Human-readable text.
    #[serde(rename = "text")]
    Text,
    /// Output as json.
    #[serde(rename = "json")]
    Json,
    /// Output as the bunyan format.
    #[serde(rename = "bunyan")]
    Bunyan,
}

/// Logging settings.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct LoggingSettings {
    /// Logging format.
    pub format: LogFormat,
    /// Whether to enable tokio console.
    pub tokio_console: bool,
    /// Whether to enable opentelemetry.
    pub opentelemetry: bool,
}

/// Retrieve [`Settings`] from the default configuration file.
#[tracing::instrument]
pub fn load_configuration() -> Result<Settings, AppError> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .add_source(config::Environment::with_prefix("app").separator("_"))
        .build()?
        .try_deserialize()?;
    Ok(settings)
}

impl DatabaseSettings {
    /// Constructs a connection string from the [`DatabaseSettings`].
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    /// Constructs a connection string from the [`DatabaseSettings`], but without the database.
    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}
