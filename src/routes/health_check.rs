use actix_web::{HttpResponse, Responder};

#[actix_web::get("/health_check")]
pub async fn health_check() -> impl Responder {
    log::info!("I'm fine!");
    HttpResponse::Ok()
}
