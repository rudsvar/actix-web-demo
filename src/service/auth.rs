//! A service that can receive user information and validate it.

use actix_http::{header::Header, StatusCode};
use actix_web::{
    error::InternalError,
    web::{self, Data},
    Error, FromRequest, HttpResponse,
};
use actix_web_httpauth::{
    extractors::basic::BasicAuth,
    headers::authorization::{Authorization, Basic},
};
use chrono::{Duration, Utc};
use futures::{future, FutureExt};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    error::BusinessError,
    middleware::security::Role,
    service::{user::user_db, ServiceResponse},
    DbPool,
};

/// A guarantee that the credentials of this user have been verified.
/// This type can only be created from a request with the appropriate credentials.
#[derive(Copy, Clone, Debug)]
pub struct AuthenticatedUser {
    id: i32,
}

impl AuthenticatedUser {
    /// Returns the id of the authenticated user.
    pub fn id(&self) -> &i32 {
        &self.id
    }
}

impl FromRequest for AuthenticatedUser {
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
        let username = auth.as_ref().user_id().to_string();
        let password = auth.as_ref().password().unwrap().to_string();

        // Get database connection
        let conn = match req.app_data::<web::Data<DbPool>>() {
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

        async move {
            let is_valid_result = user_db::authenticate(&conn, &username, &password).await;
            match is_valid_result {
                Ok(Some(id)) => Ok(AuthenticatedUser { id }),
                Ok(None) => {
                    Err(InternalError::new("Unauthorized", StatusCode::UNAUTHORIZED).into())
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    id: i32,
    exp: usize,
    roles: Vec<Role>,
}

#[actix_web::post("/login")]
pub async fn login(pool: Data<DbPool>, credentials: BasicAuth) -> ServiceResponse {
    tracing::debug!("Logging in user {}", credentials.user_id());

    // Load user information
    let password = credentials
        .password()
        .ok_or(BusinessError::AuthenticationError)?;

    // Authenticate user
    let user_id = user_db::authenticate(pool.get_ref(), credentials.user_id(), password)
        .await?
        .ok_or(BusinessError::AuthenticationError)?;

    // Fetch user roles
    let roles = user_db::get_roles(pool.get_ref(), credentials.user_id()).await?;

    // Set claims
    let in_one_minute = Utc::now() + Duration::minutes(1);
    let exp = in_one_minute.naive_utc().timestamp();
    let claims = Claims {
        id: user_id,
        exp: exp as usize,
        roles,
    };

    // Read secret from config
    let config = crate::configuration::get_configuration()?;

    // Create jwt
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .unwrap();

    tracing::info!("Sent jwt to: {}", user_id);

    Ok(HttpResponse::Created().body(token))
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
