#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    missing_docs
)]

//! A demo web service implemented with actix web.

use crate::middleware::{DigestFilter, SignatureFilter};
use crate::security::jwt::Role;
use actix_cors::Cors;
use actix_web::HttpResponse;
use actix_web::{dev::Server, web, App, HttpServer};
use actix_web_grants::proc_macro::has_roles;
use actix_web_httpauth::middleware::HttpAuthentication;
use error::AppError;
use graphql::schema::create_schema;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use service::{
    client_context::client_context,
    health_check::health,
    token::{request_token, verify_token},
};
use sqlx::{PgPool, Postgres, Transaction};
use std::io::{self};
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
    let ssl_builder = ssl_builder();
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
            .service(health)
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
                    .wrap(DigestFilter)
                    .route("", web::get().to(HttpResponse::Ok)),
            )
    })
    .listen(http_listener)?
    .listen_openssl(https_listener, ssl_builder)?
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

fn ssl_builder() -> SslAcceptorBuilder {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("test-key.pem", SslFiletype::PEM)
        .expect("failed to open/read test-key.pem");
    builder
        .set_certificate_chain_file("test-cert.pem")
        .expect("failed to open/read test-cert.pem");
    builder
}
