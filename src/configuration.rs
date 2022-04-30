//! Structs and functions for reading application configuration from a file.

/// Application settings.
#[derive(Debug, serde:: Deserialize)]
pub struct Settings {
    /// The application port.
    pub application_port: u16,
    /// JSON Web Token secret
    pub jwt_secret: String,
    /// The database settings.
    pub database: DatabaseSettings,
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
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialise our configuration reader
    let settings = config::Config::builder()
        // Add configuration values from a file named `configuration`.
        // It will look for any top-level file with an extension
        // that `config` knows how to parse: yaml, json, etc.
        .add_source(config::File::with_name("configuration"))
        .build()?;
    // Try to convert the configuration values it read into
    // our Settings type
    settings.try_deserialize().map_err(|e| {
        tracing::error!("{}", e);
        e
    })
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
