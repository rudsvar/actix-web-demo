use actix_web_demo::{
    configuration::{load_configuration, DatabaseSettings},
    telemetry, DbPool,
};
use once_cell::sync::Lazy;
use sqlx::Executor;
use std::net::TcpListener;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber_name = "test".to_string();
    let default_filter_level = "info".to_string();
    if std::env::var("RUST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    };
});

pub struct TestApp {
    address: String,
    db: DbPool,
}

impl TestApp {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn db(&self) -> &DbPool {
        &self.db
    }
}

/// Sets up a database connection pool for testing.
/// This will connect using the provided database settings,
/// create a new logical database, and run all migrations on it.
pub async fn test_db(mut database_settings: DatabaseSettings) -> DbPool {
    // Generate random database name and connection string
    let database_name = Uuid::new_v4().to_string();
    database_settings.database_name = database_name.clone();

    // Connect to database
    let db = DbPool::connect(&database_settings.connection_string_without_db())
        .await
        .expect("could not connect to db");

    // Create logical database
    db.execute(format!(r#"CREATE DATABASE "{}";"#, database_name).as_str())
        .await
        .expect("could not create database");

    // Connect to logical database
    let db = DbPool::connect(&database_settings.connection_string())
        .await
        .expect("could not connect to db");

    // Migrate database
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("could not migrate the database");

    db
}

// Spawn an application used for testing
pub async fn spawn_test_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = load_configuration().expect("Failed to read configuration");
    let db = test_db(configuration.database).await;
    let server = actix_web_demo::run_app(listener, db.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp { address, db }
}
