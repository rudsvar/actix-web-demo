//! A service that can receive user information and validate it.

use crate::{
    infra::{error::AppError, security::jwt::Claims},
    security::jwt::{decode_jwt, encode_jwt},
    AppResult, DbPool,
};
use actix_web::{web::Data, HttpResponse};
use actix_web_httpauth::extractors::{basic::BasicAuth, bearer::BearerAuth};
use http::StatusCode;

#[actix_web::post("/token")]
#[tracing::instrument(skip_all, fields(username = credentials.user_id()))]
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

/// A basic authentication header value.
#[derive(Clone, Debug)]
pub struct BasicAuth2 {
    user_id: String,
    password: String,
}

#[async_trait::async_trait]
impl<B> axum::extract::FromRequest<B> for BasicAuth2
where
    B: Send, // required by `async_trait`
{
    type Rejection = (axum::http::StatusCode, String);

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let header = req.headers().get("authorization").ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing authorization header".to_string(),
        ))?;
        let header = header.to_str().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Header not valid ASCII".to_string(),
            )
        })?;
        let header = header.strip_prefix("Basic ").ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid authorization header prefix".to_string(),
        ))?;
        let header = base64::decode(header).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid base64 encoding of basic auth".to_string(),
            )
        })?;
        let header = String::from_utf8(header).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Base64 decoded basic auth header is not valid UTF-8".to_string(),
            )
        })?;
        let (user_id, password) = header.split_once(':').ok_or((
            StatusCode::BAD_REQUEST,
            "Basic auth header must contain username and password".to_string(),
        ))?;
        Ok(Self {
            user_id: user_id.to_string(),
            password: password.to_string(),
        })
    }
}

/// Creates a JWT for a user authenticating themselves.
#[tracing::instrument(skip_all, fields(username = credentials.user_id))]
pub async fn request_token2(
    axum::Extension(pool): axum::Extension<DbPool>,
    credentials: BasicAuth2,
) -> axum::response::Result<String> {
    // Load user information
    let username = credentials.user_id;
    tracing::debug!("Token requested by `{}`", username);
    let password = credentials.password;

    let token = encode_jwt(&pool, &username, &password).await?;
    tracing::debug!("Sending token to `{}`", username);

    Ok(token)
}

#[actix_web::get("/verify")]
pub async fn verify_token(auth: BearerAuth) -> AppResult<HttpResponse> {
    tracing::debug!("Verifying jwt");
    let token = auth.token();
    let claims = decode_jwt(token)?;
    tracing::debug!("Got claims {:?}", claims);
    Ok(HttpResponse::Ok().json(claims))
}

/// A bearer token header value.
#[derive(Clone, Debug)]
pub struct BearerAuth2 {
    token: String,
}

#[async_trait::async_trait]
impl<B> axum::extract::FromRequest<B> for BearerAuth2
where
    B: Send, // required by `async_trait`
{
    type Rejection = axum::http::StatusCode;

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let header = req
            .headers()
            .get("authorization")
            .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;
        let header = header
            .to_str()
            .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
        let token = header
            .strip_prefix("Bearer ")
            .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;
        Ok(Self {
            token: token.to_string(),
        })
    }
}

/// Verifies a JWT.
#[tracing::instrument(skip_all)]
pub async fn verify_token2(auth: BearerAuth2) -> axum::response::Result<axum::Json<Claims>> {
    tracing::debug!("Verifying jwt");
    let token = auth.token;
    let claims = decode_jwt(&token)?;
    tracing::debug!("Got claims {:?}", claims);
    Ok(axum::Json(claims))
}
