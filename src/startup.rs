use crate::routes::{client_context, health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use r2d2_sqlite::SqliteConnectionManager;
use std::net::TcpListener;

pub type DbPool = r2d2::Pool<SqliteConnectionManager>;

pub fn run(listener: TcpListener) -> std::io::Result<Server> {
    let manager = SqliteConnectionManager::memory();
    let db_pool = DbPool::builder()
        .build(manager)
        .expect("could not initialize db");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .service(health_check)
            .service(subscribe)
            .route("/client_context", web::get().to(client_context))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
