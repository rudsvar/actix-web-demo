//! A service that can receive user information and validate it.

use crate::{
    error::AppError,
    security::jwt::{decode_jwt, encode_jwt},
    AppResult, DbPool,
};
use actix_web::{web::Data, HttpResponse};
use actix_web_httpauth::extractors::{basic::BasicAuth, bearer::BearerAuth};

#[actix_web::post("/token")]
pub async fn request_token(pool: Data<DbPool>, credentials: BasicAuth) -> AppResult<HttpResponse> {
    // Load user information
    let username = credentials.user_id();
    tracing::debug!("Token requested by `{}`", credentials.user_id());
    let password = credentials
        .password()
        .ok_or(AppError::AuthenticationError)?;

    let token = encode_jwt(pool.get_ref(), username, password).await?;
    tracing::debug!("Sending token to `{}`", credentials.user_id());

    Ok(HttpResponse::Created().body(token))
}

#[actix_web::get("/verify")]
pub async fn verify_token(auth: BearerAuth) -> AppResult<HttpResponse> {
    tracing::debug!("Verifying jwt");
    let token = auth.token();
    let claims = decode_jwt(token)?;
    tracing::debug!("Got claims {:?}", claims);
    Ok(HttpResponse::Ok().json(claims))
}
