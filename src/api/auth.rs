//! A service that can receive user information and validate it.

use std::str::FromStr;

use actix_http::{header::Header, StatusCode};
use actix_web::{error::InternalError, web, Error, FromRequest, HttpResponse};
use actix_web_httpauth::headers::authorization::{Authorization, Basic};
use chrono::{Duration, Utc};
use futures::{future, FutureExt};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// A guarantee that the credentials of this user have been verified.
/// This type can only be created from a request with the appropriate credentials.
#[derive(Copy, Clone, Debug)]
pub struct BasicAuth {
    id: Uuid,
}

impl BasicAuth {
    /// Returns the id of the authenticated user.
    pub fn id(&self) -> &Uuid {
        &self.id
    }
}

impl FromRequest for BasicAuth {
    type Error = Error;
    type Future = future::LocalBoxFuture<'static, Result<Self, Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_http::Payload) -> Self::Future {
        let auth = Authorization::<Basic>::parse(req);
        if let Err(e) = auth {
            return async move {
                Err(
                    InternalError::new(format!("Can't authorize: {}", e), StatusCode::UNAUTHORIZED)
                        .into(),
                )
            }
            .boxed_local();
        }
        let auth = auth.unwrap();
        let id = auth.as_ref().user_id();
        let password = auth.as_ref().password().unwrap();

        // Get database connection
        let conn = match req.app_data::<web::Data<PgPool>>() {
            Some(conn) => conn.get_ref().clone(),
            None => {
                return async {
                    Err(InternalError::new(
                        "Can't connect to database",
                        StatusCode::SERVICE_UNAVAILABLE,
                    )
                    .into())
                }
                .boxed_local()
            }
        };

        let id = id.to_string();
        let id = Uuid::from_str(&id).unwrap();
        let password = password.to_string();

        async move {
            let is_valid_result = crate::db::user::verify_password(&conn, &id, &password).await;
            match is_valid_result {
                Ok(is_valid) => {
                    if is_valid {
                        Ok(BasicAuth { id })
                    } else {
                        Err(InternalError::new("Unauthorized", StatusCode::UNAUTHORIZED).into())
                    }
                }
                Err(e) => {
                    Err(InternalError::new(e.to_string(), StatusCode::SERVICE_UNAVAILABLE).into())
                }
            }
        }
        .boxed_local()
    }
}

/// The data stored in the jwt
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    id: Uuid,
    exp: usize,
}

#[actix_web::post("/login")]
pub async fn login(user: BasicAuth) -> HttpResponse {
    // Read secret from config
    let config = match crate::configuration::get_configuration() {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    // Set claims
    let in_one_minute = Utc::now() + Duration::seconds(60);
    let exp = in_one_minute.naive_utc().timestamp();
    let claims = Claims {
        id: *user.id(),
        exp: exp as usize,
    };

    // Create jwt
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .unwrap();

    tracing::info!("Sent jwt to: {}", user.id);

    HttpResponse::Created().body(token)
}

#[actix_web::post("/verify")]
pub async fn verify(token: web::Json<String>) -> HttpResponse {
    tracing::info!("Token {}", token);
    let config = match crate::configuration::get_configuration() {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    let decoded = jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    );
    match decoded {
        Ok(token) => {
            tracing::info!("Header {:?}", token.header);
            tracing::info!("Claims {:?}", token.claims);
            HttpResponse::Ok().body("Ok")
        }
        Err(e) => HttpResponse::Forbidden().body(format!("Forbidden: {}", e)),
    }
}
