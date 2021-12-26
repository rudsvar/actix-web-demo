use crate::routes::{client_context, health_check, post_subscription};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
// use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, db_pool: PgPool) -> std::io::Result<Server> {
    let pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            // .wrap(TracingLogger::default())
            .app_data(pool.clone())
            .service(health_check)
            .service(post_subscription)
            .route("/client_context", web::get().to(client_context))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
