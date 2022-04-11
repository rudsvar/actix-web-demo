use actix_web_demo::{configuration::get_configuration, telemetry, DbPool};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber =
        telemetry::get_subscriber("actix-web-demo".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration = get_configuration().expect("could not read configuration");

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
