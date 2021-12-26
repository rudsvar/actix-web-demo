use actix_web_demo::{configuration::get_configuration, startup};
use env_logger::Env;
use sqlx::{migrate::Migrator, PgPool};
use std::net::TcpListener;

static MIGRATOR: Migrator = sqlx::migrate!();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = get_configuration().expect("could not read configuration");

    // Connect to db
    let connection_string = configuration.database.connection_string();
    let db_pool = PgPool::connect(&connection_string)
        .await
        .expect("could not connect to db");
    MIGRATOR
        .run(&db_pool)
        .await
        .expect("could not run migrations");

    // Create listener
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    // Start application
    startup::run(listener, db_pool)?.await
}
