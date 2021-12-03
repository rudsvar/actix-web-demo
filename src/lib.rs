use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::net::TcpListener;

#[actix_web::get("/health_check")]
pub async fn health_check() -> impl Responder {
    log::info!("I'm fine!");
    HttpResponse::Ok()
}

#[derive(Debug, Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[actix_web::post("/subscriptions")]
pub async fn subscribe(_form: web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(|| App::new().service(health_check).service(subscribe))
        .listen(listener)?
        .run();
    Ok(server)
}
