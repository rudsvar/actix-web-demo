//! Health check related routes.

use actix_web::{HttpResponse, Responder};

#[actix_web::get("/health")]
#[tracing::instrument]
pub async fn health() -> impl Responder {
    tracing::info!("I'm fine!");
    HttpResponse::Ok().finish()
}

/// Health check.
#[tracing::instrument]
pub async fn health2() {
    tracing::info!("I'm fine!");
}
