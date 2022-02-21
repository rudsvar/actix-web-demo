//! A function for starting a server.

use crate::api::{
    client_context::client_context, health_check::health_check, subscription::*, user::*,
};
use crate::middleware;
use crate::service::login::login;
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

/// Starts a [`Server`].
pub fn run(listener: TcpListener, db_pool: PgPool) -> std::io::Result<Server> {
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
                    .service(login),
            )
            // Other
            .service(client_context)
    })
    .listen(listener)?
    .run();
    Ok(server)
}
