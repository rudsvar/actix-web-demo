use crate::routes::{client_context, health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use std::net::TcpListener;

pub fn run(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .service(health_check)
            .service(subscribe)
            .route("/client_context", web::get().to(client_context))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
