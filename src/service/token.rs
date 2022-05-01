//! A service that can receive user information and validate it.

use super::AppResult;
use crate::{
    error::BusinessError,
    security::{decode_jwt, encode_jwt},
    DbPool,
};
use actix_web::{web::Data, HttpResponse};
use actix_web_httpauth::extractors::{basic::BasicAuth, bearer::BearerAuth};

#[actix_web::post("/login")]
pub async fn login(pool: Data<DbPool>, credentials: BasicAuth) -> AppResult<HttpResponse> {
    tracing::debug!("Logging in user {}", credentials.user_id());

    // Load user information
    let username = credentials.user_id();
    let password = credentials
        .password()
        .ok_or(BusinessError::AuthenticationError)?;

    let token = encode_jwt(pool.get_ref(), username, password).await?;

    tracing::info!("Sent jwt to: {}", username);

    Ok(HttpResponse::Created().body(token))
}

#[actix_web::get("/verify")]
pub async fn verify(auth: BearerAuth) -> AppResult<HttpResponse> {
    let token = auth.token();
    tracing::debug!("Got token {}", token);
    let claims = decode_jwt(token)?;
    tracing::debug!("Got claims {:?}", claims);
    Ok(HttpResponse::Ok().json(claims))
}
