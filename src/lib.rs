#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    missing_docs
)]

//! A demo web service implemented with actix web.

use crate::api::{
    client_context::client_context, health_check::health_check, subscription::*, user::*,
};
use crate::service::auth::{login, verify};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::io;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub mod api;
pub mod configuration;
pub mod db;
pub mod middleware;
pub mod model;
pub mod service;
pub mod telemetry;

/// Starts a [`Server`].
pub fn run_app(listener: TcpListener, db_pool: PgPool) -> io::Result<Server> {
    let pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            // Database pool
            .app_data(pool.clone())
            // Middleware to apply to all requests
            .wrap(TracingLogger::default())
            .wrap(middleware::ResponseAppender)
            // Health check
            .service(health_check)
            .service(
                // Subscription
                web::scope("/api")
                    .service(post_subscription)
                    .service(put_subscription)
                    .service(list_subscriptions)
                    .service(post_user)
                    .service(get_user)
                    .service(list_users)
                    .service(login)
                    .service(verify),
            )
            // Other
            .service(client_context)
    })
    .listen(listener)?
    .run();
    Ok(server)
}
