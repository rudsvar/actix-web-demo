use actix_web_demo::{infra::configuration::load_configuration, DbPool};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    actix_web_demo::infra::logging::init_logging();

    let configuration = load_configuration().expect("could not read configuration");

    // Connect to db
    let connection_string = configuration.database.connection_string();
    let db_pool = DbPool::connect_lazy(&connection_string).expect("could not connect to db");

    let grpc = actix_web_demo::run_grpc("0.0.0.0:3009".parse().unwrap(), db_pool.clone());
    tokio::spawn(grpc);

    // Create http listener
    let http_addr = format!(
        "{}:{}",
        configuration.server.address, configuration.server.http_port
    );
    let http_listener = TcpListener::bind(http_addr)?;

    // Create https listener
    let https_addr = format!(
        "{}:{}",
        configuration.server.address, configuration.server.https_port
    );
    let https_listener = TcpListener::bind(https_addr)?;

    // Start application
    actix_web_demo::run_app(http_listener, https_listener, db_pool)?.await
}
