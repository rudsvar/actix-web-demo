//! Types and functions for setting up application security.

use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::{repository::user_repository, error::AppError, DbPool};
use actix_http::{HttpMessage, StatusCode};
use actix_web::{dev::ServiceRequest, Error};
use actix_web_grants::permissions::AttachPermissions;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use tonic::{Request, Status};

/// The data stored in the jwt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: i32,
    exp: usize,
    roles: Vec<Role>,
}

impl Claims {
    /// Returns the roles stored in the claim.
    pub fn id(&self) -> i32 {
        self.sub
    }
    /// Returns the roles stored in the claim.
    pub fn roles(&self) -> &[Role] {
        &self.roles
    }
    /// Check if the claim contains a specific role.
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
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

fn file_to_bytes(path: &str) -> Result<Vec<u8>, AppError> {
    let mut buf = Vec::new();
    let file = File::open(path).map_err(|_| {
        AppError::CustomError(
            format!("failed to open {}", path),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;
    let mut file = BufReader::new(file);
    file.read_to_end(&mut buf).map_err(|_| {
        AppError::CustomError(
            format!("failed to read {}", path),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;
    Ok(buf)
}

/// Create a jwt for the provided user.
pub async fn encode_jwt(conn: &DbPool, username: &str, password: &str) -> Result<String, AppError> {
    // Authenticate user
    let user_id = user_repository::authenticate(conn, username, password)
        .await?
        .ok_or(AppError::AuthenticationError)?;

    // Fetch user roles
    let roles = user_repository::fetch_roles(conn, username).await?;

    let config = crate::configuration::load_configuration()?;

    // Set claims
    let in_one_minute = Utc::now() + Duration::minutes(config.security.jwt_minutes_to_live);
    let exp = in_one_minute.naive_utc().timestamp();
    let claims = Claims {
        sub: user_id,
        exp: exp as usize,
        roles,
    };

    // Read secret from config
    let private_key_path = config.security.jwt_private_key;
    let private_key = file_to_bytes(&private_key_path)?;
    let encoding_key = EncodingKey::from_ec_pem(&private_key).map_err(|e| {
        AppError::CustomError(
            format!("failed to create encoding key: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // Create jwt
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::ES256),
        &claims,
        &encoding_key,
    )
    .map_err(|_| AppError::AuthenticationError)?;

    Ok(token)
}

/// Decode a jwt into its claims.
pub fn decode_jwt(token: &str) -> Result<Claims, AppError> {
    // Read secret from config
    let config = crate::configuration::load_configuration()?;
    let public_key_path = config.security.jwt_public_key;
    let public_key = file_to_bytes(&public_key_path)?;
    let decoding_key = DecodingKey::from_ec_pem(&public_key).map_err(|_| {
        AppError::CustomError(
            "failed to create decoding key".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    let decoded =
        jsonwebtoken::decode::<Claims>(token, &decoding_key, &Validation::new(Algorithm::ES256))
            .map_err(|_| AppError::AuthenticationError)?;
    Ok(decoded.claims)
}

/// A validator for [`actix_web_httpauth::middleware::HttpAuthentication`] that gets roles from a JWT.
///
/// # Examples
///
/// ```
/// # use actix_web_httpauth::middleware::HttpAuthentication;
/// # use actix_web_demo::security::jwt::validate_jwt;
/// let auth = HttpAuthentication::bearer(validate_jwt);
/// ```
pub async fn validate_jwt(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    if let Ok(claims) = decode_jwt(token) {
        tracing::debug!("Found claims: {:?}", claims);
        req.attach(claims.roles().to_vec());
        req.extensions_mut().insert(claims);
        Ok(req)
    } else {
        Err((AppError::AuthenticationError.into(), req))
    }
}

/// Check that the incoming gRPC request contains a valid jwt.
pub fn jwt_interceptor(mut request: Request<()>) -> Result<Request<()>, Status> {
    tracing::debug!("Checking JWT");

    // Get authorization header
    let token = request
        .metadata()
        .get("authorization")
        .ok_or_else(|| Status::unauthenticated("missing authorization header"))?;

    // Convert header to str
    let token = token
        .to_str()
        .map_err(|e| Status::invalid_argument(format!("invalid authorization header: {}", e)))?;

    // Split into token type and header
    let (token_type, token) = token
        .split_once(' ')
        .ok_or_else(|| Status::invalid_argument("invalid authorization header format"))?;

    match token_type {
        "Bearer" => {
            let claims = decode_jwt(token)?;
            tracing::debug!("Found claims: {:?}", claims);
            request.extensions_mut().insert(claims);
            Ok(request)
        }
        _ => Err(Status::unauthenticated("unknown token type")),
    }
}
