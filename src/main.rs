use actix_web_demo::infra::configuration::load_configuration;
use sqlx::{
    pool::PoolOptions,
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};
use std::{net::TcpListener, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = load_configuration()?;

    actix_web_demo::infra::logging::init_logging(&configuration)?;

    // Configure database connection
    let mut db_options = PgConnectOptions::default()
        .application_name(&configuration.application.name)
        .host(&configuration.database.host)
        .username(&configuration.database.username)
        .password(&configuration.database.password)
        .port(configuration.database.port)
        .ssl_mode(PgSslMode::Prefer);
    db_options.log_statements(tracing::log::LevelFilter::Trace);
    let db_pool = PoolOptions::default()
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(db_options);

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .unwrap_or_else(|e| tracing::error!("Failed to run migrations: {}", e));

    let grpc_addr = format!(
        "{}:{}",
        configuration.server.grpc_address, configuration.server.grpc_port
    )
    .parse()?;
    let grpc = actix_web_demo::run_grpc(grpc_addr, db_pool.clone());
    tokio::spawn(grpc);

    // Create http listener
    let http_addr = format!(
        "{}:{}",
        configuration.server.address, configuration.server.http_port
    );
    let http_listener = TcpListener::bind(http_addr)?;

    // Start application
    actix_web_demo::run_actix(http_listener, db_pool)?.await?;

    Ok(())
}
