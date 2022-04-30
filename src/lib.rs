#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    missing_docs
)]

//! A demo web service implemented with actix web.

use actix_web::HttpResponse;
use actix_web::{dev::Server, web, App, HttpServer};
use actix_web_grants::proc_macro::has_permissions;
use actix_web_grants::GrantsMiddleware;
use middleware::security::grants_extractor;
use service::user::user_api::{get_user, list_users, post_user};
use service::{
    account::{deposit, get_account, post_account, transfer, withdraw},
    auth::{login, verify},
    client_context::client_context,
    health_check::health_check,
};
use sqlx::PgPool;
use std::io;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub mod configuration;
pub mod error;
pub mod middleware;
pub mod service;
pub mod telemetry;
pub mod validated;

/// The database connection pool type used in the application.
pub type DbPool = PgPool;

/// Starts a [`Server`].
pub fn run_app(listener: TcpListener, db_pool: DbPool) -> io::Result<Server> {
    let pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            // Database pool
            .app_data(pool.clone())
            // Middleware to apply to all requests
            .wrap(TracingLogger::default())
            .wrap(middleware::ResponseAppender)
            .wrap(GrantsMiddleware::with_extractor(grants_extractor))
            // Health check
            .service(health_check)
            .service(
                // Subscription
                web::scope("/api")
                    .service(post_user)
                    .service(get_user)
                    .service(list_users)
                    .service(login)
                    .service(verify)
                    .service(post_account)
                    .service(get_account)
                    .service(deposit)
                    .service(withdraw)
                    .service(transfer),
            )
            // Other
            .service(client_context)
            // Secure endpoints
            .route("/user", web::get().to(user))
            .route("/admin", web::get().to(admin))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

#[has_permissions("USER")]
async fn user() -> HttpResponse {
    HttpResponse::Ok().body("Hello user!".to_string())
}

#[has_permissions("ADMIN")]
async fn admin() -> HttpResponse {
    HttpResponse::Ok().body("Hello admin!".to_string())
}
