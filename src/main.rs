use actix_web_demo::{configuration::load_configuration, DbPool};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let configuration = load_configuration().expect("could not read configuration");

    // Connect to db
    let connection_string = configuration.database.connection_string();
    let db_pool = DbPool::connect(&connection_string)
        .await
        .expect("could not connect to db");

    // Create listener
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    // Start application
    actix_web_demo::run_app(listener, db_pool)?.await
}
