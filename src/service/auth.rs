//! A service that can receive user information and validate it.

use actix_http::{header::Header, StatusCode};
use actix_web::{
    error::InternalError,
    web::{self, Data},
    Error, FromRequest, HttpResponse,
};
use actix_web_httpauth::{
    extractors::basic::BasicAuth,
    headers::authorization::{Authorization, Bearer},
};
use chrono::{Duration, Utc};
use futures::{future, FutureExt};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, BusinessError},
    middleware::security::Role,
    service::{user::user_db, AppResponse},
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
        let auth = Authorization::<Bearer>::parse(req);
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
        let token = auth.as_ref().token().to_string();
        let claims = decode_jwt(&token);
        if let Err(e) = claims {
            return async move {
                Err(
                    InternalError::new(format!("Can't authorize: {}", e), StatusCode::UNAUTHORIZED)
                        .into(),
                )
            }
            .boxed_local();
        }
        let claims = claims.unwrap();

        async move { Ok(AuthenticatedUser { id: claims.id }) }.boxed_local()
    }
}

/// The data stored in the jwt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    id: i32,
    exp: usize,
    roles: Vec<Role>,
}

impl Claims {
    /// Returns the roles stored in the claim.
    pub fn roles(&self) -> &[Role] {
        &self.roles
    }
}

#[actix_web::post("/login")]
pub async fn login(pool: Data<DbPool>, credentials: BasicAuth) -> AppResponse {
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

#[actix_web::post("/verify")]
pub async fn verify(token: web::Json<String>) -> AppResponse {
    tracing::debug!("Token {}", token);
    let claims = decode_jwt(token.as_str())?;
    tracing::debug!("Got claims {:?}", claims);
    Ok(HttpResponse::Ok().json(claims))
}

/// Create a jwt for the provided user.
pub async fn encode_jwt(conn: &DbPool, username: &str, password: &str) -> Result<String, AppError> {
    // Authenticate user
    let user_id = user_db::authenticate(conn, username, password)
        .await?
        .ok_or(BusinessError::AuthenticationError)?;

    // Fetch user roles
    let roles = user_db::fetch_roles(conn, username).await?;

    // Set claims
    let in_one_minute = Utc::now() + Duration::minutes(1);
    let exp = in_one_minute.naive_utc().timestamp();
    let claims = Claims {
        id: user_id,
        exp: exp as usize,
        roles,
    };

    // Read secret from config
    let config = crate::configuration::load_configuration()?;

    // Create jwt
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .unwrap();

    Ok(token)
}

/// Decode a jwt into its claims.
pub fn decode_jwt(token: &str) -> Result<Claims, AppError> {
    let config = crate::configuration::load_configuration()?;
    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| BusinessError::AuthenticationError)?;
    Ok(decoded.claims)
}
