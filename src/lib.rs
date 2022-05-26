#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    missing_docs
)]

//! A demo web service implemented with actix web.

use crate::middleware::SignatureFilter;
use crate::security::jwt::Role;
use actix_cors::Cors;
use actix_web::HttpResponse;
use actix_web::{dev::Server, web, App, HttpServer};
use actix_web_grants::proc_macro::has_roles;
use actix_web_httpauth::middleware::HttpAuthentication;
use error::AppError;
use graphql::schema::create_schema;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use service::{
    client_context::client_context,
    health_check::health_check,
    token::{request_token, verify_token},
};
use sqlx::{PgPool, Postgres, Transaction};
use std::fs::File;
use std::io::{self, BufReader};
use std::net::TcpListener;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

pub mod configuration;
pub mod error;
pub mod graphql;
pub mod middleware;
pub mod security;
pub mod service;
pub mod validation;

/// A common response type for services.
pub type AppResult<T> = Result<T, AppError>;

/// The database connection pool type used in the application.
pub type DbPool = PgPool;

/// The database transaction type.
pub type Tx = Transaction<'static, Postgres>;

/// Starts a [`Server`].
pub fn run_app(
    http_listener: TcpListener,
    https_listener: TcpListener,
    db_pool: DbPool,
) -> io::Result<Server> {
    tracing::info!(
        "Starting application on address {} and {}",
        http_listener.local_addr().unwrap(),
        https_listener.local_addr().unwrap()
    );
    let config = load_rustls_config();
    let pool = web::Data::new(db_pool.clone());
    let schema = Arc::new(create_schema(db_pool));
    let server = HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(security::jwt::validate_jwt);
        App::new()
            // Database pool
            .app_data(pool.clone())
            // Middleware to apply to all requests
            .wrap(middleware::RequestWrapper)
            .wrap(TracingLogger::default())
            // GraphQL
            .app_data(web::Data::from(schema.clone()))
            .service(graphql::graphql)
            .service(graphql::graphql_playground)
            .wrap(Cors::permissive())
            // Health check
            .service(health_check)
            .service(request_token)
            .service(verify_token)
            // Other
            .service(client_context)
            // Api
            .service(
                web::scope("/api")
                    .wrap(auth)
                    .configure(service::account::account_api::account_config)
                    .configure(service::user::user_api::user_config)
                    .configure(service::transfer::transfer_api::transfer_config)
                    // Secure endpoints
                    .route("/user", web::get().to(user))
                    .route("/admin", web::get().to(admin)),
            )
            .service(
                web::scope("/signature")
                    .wrap(SignatureFilter)
                    .route("", web::get().to(HttpResponse::Ok)),
            )
    })
    .listen(http_listener)?
    .listen_rustls(https_listener, config)?
    // .bind_rustls("127.0.0.1:8443", config)?
    .run();
    Ok(server)
}

#[has_roles("Role::User", type = "Role")]
async fn user() -> HttpResponse {
    HttpResponse::Ok().body("Hello user!".to_string())
}

#[has_roles("Role::Admin", type = "Role")]
async fn admin() -> HttpResponse {
    HttpResponse::Ok().body("Hello admin!".to_string())
}

fn load_rustls_config() -> rustls::ServerConfig {
    // init server config builder with safe defaults
    let rustls_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    let app_config =
        crate::configuration::load_configuration().expect("could not load configuration");

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(File::open(app_config.security.certificate).unwrap());
    let key_file = &mut BufReader::new(File::open(app_config.security.private_key).unwrap());

    // convert files to key/cert objects
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    rustls_config
        .with_single_cert(cert_chain, keys.remove(0))
        .unwrap()
}
