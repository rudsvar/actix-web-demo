//! Types and functions for setting up application security.

use crate::service::auth::decode_jwt;
use actix_web::{dev::ServiceRequest, Error};
use actix_web_grants::permissions::AttachPermissions;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};

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
