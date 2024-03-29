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

    // Configure database connection
    let mut db_options = PgConnectOptions::default()
        .application_name(&configuration.application.name)
        .host(&configuration.database.host)
        .username(&configuration.database.username)
        .password(&configuration.database.password)
        .database(&configuration.database.database_name)
        .port(configuration.database.port)
        .ssl_mode(PgSslMode::Prefer);
    db_options.log_statements(tracing::log::LevelFilter::Debug);
    let db_pool = PoolOptions::default()
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(db_options);

    actix_web_demo::infra::logging::init_logging(&configuration, db_pool.clone()).await?;

    // Run migrations
    while let Err(e) = sqlx::migrate!("./migrations").run(&db_pool).await {
        tracing::error!("Failed to run migrations: {}", e);
        tokio::time::sleep(Duration::from_secs(30)).await;
    }

    let grpc_addr = format!(
        "{}:{}",
        configuration.server.grpc_address, configuration.server.grpc_port
    )
    .parse()?;
    let grpc = actix_web_demo::run_grpc(grpc_addr, db_pool.clone());
    tokio::spawn(grpc);

    tokio::spawn(actix_web_demo::run_axum(
        "0.0.0.0:8081".parse()?,
        db_pool.clone(),
    ));

    // Create http listener
    let http_addr = format!(
        "{}:{}",
        configuration.server.address, configuration.server.http_port
    );
    let http_listener = TcpListener::bind(http_addr)?;

    // Start application
    actix_web_demo::run_actix(http_listener, db_pool.clone())?.await?;

    Ok(())
}
