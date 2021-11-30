use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

#[actix_web::get("/health_check")]
pub async fn health_check(_: HttpRequest) -> impl Responder {
    log::info!("I'm fine!");
    HttpResponse::Ok()
}

pub async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

pub fn run() -> std::io::Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .service(health_check)
            .route("/hello/", web::get().to(greet))
            .route("/hello/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8080")?
    .run();
    Ok(server)
}
