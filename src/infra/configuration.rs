//! Structs and functions for reading application configuration from a file.

use super::error::AppError;

/// Application settings.
#[derive(Debug, serde:: Deserialize)]
pub struct Settings {
    /// Server settings.
    pub server: ServerSettings,
    /// Security settings.
    pub security: SecuritySettings,
    /// Database settings.
    pub database: DatabaseSettings,
}

/// Server settings.
#[derive(Clone, Debug, serde:: Deserialize)]
pub struct ServerSettings {
    /// Server address.
    pub address: String,
    /// Server http port.
    pub http_port: u16,
    /// Server https port.
    pub https_port: u16,
}

/// Security settings.
#[derive(Clone, Debug, serde:: Deserialize)]
pub struct SecuritySettings {
    /// SSL certificate.
    pub ssl_certificate: String,
    /// SSL private key.
    pub ssl_private_key: String,
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
#[derive(Debug, serde:: Deserialize)]
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

/// Retrieve [`Settings`] from the default configuration file.
pub fn load_configuration() -> Result<Settings, AppError> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration"))
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
