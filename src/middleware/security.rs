//! Types and functions for setting up application security.

use crate::{service::user::user_db, DbPool};
use actix_web::{dev::ServiceRequest, web::Data, Error};
use actix_web_grants::permissions::AttachPermissions;
use actix_web_httpauth::extractors::basic::BasicAuth;
use serde::{Deserialize, Serialize};

/// The possible roles used in the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "role_name")]
pub enum Role {
    /// Administrator with all privileges.
    Admin,
    /// User with access to their own data.
    User,
}

/// Validates a request
pub async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, Error> {
    tracing::info!("Entering validator");
    let pool = req.app_data::<Data<DbPool>>().unwrap();
    let username = credentials.user_id();
    let roles = user_db::fetch_roles(pool.get_ref(), username).await.unwrap();
    req.attach(roles);
    Ok(req)
}
