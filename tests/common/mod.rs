use actix_web_demo::{
    infra::configuration::{self, load_configuration, DatabaseSettings},
    infra::logging,
    DbPool,
};
use once_cell::sync::Lazy;
use sqlx::Executor;
use std::net::TcpListener;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let config = configuration::load_configuration().unwrap();
    logging::init_logging(&config).unwrap();
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

    // Create http listener
    let http_listener = TcpListener::bind("127.0.0.1:0").unwrap();

    // Only need http for tests
    let address = format!("http://{}", http_listener.local_addr().unwrap());

    let configuration = load_configuration().expect("Failed to read configuration");
    let db = test_db(configuration.database).await;
    let server =
        actix_web_demo::run_actix(http_listener, db.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp { address, db }
}
