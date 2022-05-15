#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    missing_docs
)]

//! A demo web service implemented with actix web.

use crate::security::Role;
use actix_cors::Cors;
use actix_web::HttpResponse;
use actix_web::{dev::Server, web, App, HttpServer};
use actix_web_grants::proc_macro::has_roles;
use actix_web_httpauth::middleware::HttpAuthentication;
use graphql::schema::create_schema;
use service::{
    client_context::client_context,
    health_check::health_check,
    token::{request_token, verify_token},
};
use sqlx::PgPool;
use std::io;
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

/// The database connection pool type used in the application.
pub type DbPool = PgPool;

/// Starts a [`Server`].
pub fn run_app(listener: TcpListener, db_pool: DbPool) -> io::Result<Server> {
    let pool = web::Data::new(db_pool.clone());
    let schema = Arc::new(create_schema(db_pool));
    let server = HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(security::validate_jwt);
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
                    .configure(service::account::account_config)
                    .configure(service::user::user_config)
                    .configure(service::deposit::deposit_config)
                    .configure(service::withdrawal::withdrawal_config)
                    .configure(service::transfer::transfer_config)
                    // Secure endpoints
                    .route("/user", web::get().to(user))
                    .route("/admin", web::get().to(admin)),
            )
    })
    .listen(listener)?
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
