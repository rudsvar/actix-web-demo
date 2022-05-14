//! Validators for [`actix_web_httpauth::middleware::HttpAuthentication`].

use crate::{error::AppError, security::decode_jwt};
use actix_http::HttpMessage;
use actix_web::{dev::ServiceRequest, Error};
use actix_web_grants::permissions::AttachPermissions;
use actix_web_httpauth::extractors::bearer::BearerAuth;

/// Validates a jwt.
///
/// # Examples
///
/// ```
/// # use actix_web_httpauth::middleware::HttpAuthentication;
/// # use actix_web_demo::middleware::validate_jwt;
/// let auth = HttpAuthentication::bearer(validate_jwt);
/// ```
pub async fn validate_jwt(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let token = credentials.token();
    if let Ok(claims) = decode_jwt(token) {
        tracing::debug!("Decoded claims: {:?}", claims);
        req.attach(claims.roles().to_vec());
        req.extensions_mut().insert(claims);
        Ok(req)
    } else {
        Err(AppError::AuthenticationError.into())
    }
}
