//! Types and functions for setting up application security.

use crate::{
    error::{AppError, BusinessError},
    service::user::user_db,
    DbPool,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};

/// The data stored in the jwt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    id: i32,
    exp: usize,
    roles: Vec<Role>,
}

impl Claims {
    /// Returns the roles stored in the claim.
    pub fn id(&self) -> i32 {
        self.id
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
