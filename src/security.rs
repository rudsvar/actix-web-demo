//! Types and functions for setting up application security.

use crate::{
    error::{AppError, BusinessError},
    service::user::user_db,
    DbPool,
};
use actix_http::{header::Header, StatusCode};
use actix_web::{dev::ServiceRequest, error::InternalError, Error, FromRequest};
use actix_web_grants::permissions::AttachPermissions;
use actix_web_httpauth::{
    extractors::bearer::BearerAuth,
    headers::authorization::{Authorization, Bearer},
};
use chrono::{Duration, Utc};
use futures::{future, FutureExt};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};

/// A guarantee that the credentials of this user have been verified.
/// This type can only be created from a request with the appropriate credentials.
#[derive(Copy, Clone, Debug)]
pub struct AuthenticatedUser {
    id: i32,
}

impl AuthenticatedUser {
    /// Returns the id of the authenticated user.
    pub fn id(&self) -> i32 {
        self.id
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

/// The possible roles used in the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "role_name")]
pub enum Role {
    /// User with access to their own data.
    User,
    /// Administrator with all privileges.
    Admin,
}

/// Validates a request
pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    tracing::info!("Entering validator");
    let token = credentials.token();
    if let Ok(claims) = decode_jwt(token) {
        req.attach(claims.roles().to_vec())
    }
    Ok(req)
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
