//! Types and functions for setting up application security.

use crate::DbPool;
use actix_http::StatusCode;
use actix_web::{dev::ServiceRequest, error::InternalError, web::Data, Error};

/// Extract grants from the database.
pub async fn grants_extractor(req: &ServiceRequest) -> Result<Vec<String>, Error> {
    let db = req.app_data::<Data<DbPool>>().unwrap();

    let actual_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| InternalError::new("Please provide auth", StatusCode::UNAUTHORIZED))?
        .to_str()
        .unwrap();

    let id: i32 = actual_header.parse().unwrap();
    let roles: Vec<_> = sqlx::query!(
        r#"
        SELECT r.name
        FROM role r
        INNER JOIN user_role u_r
        ON r.id = u_r.role_id
        WHERE u_r.user_id = $1
        "#,
        id
    )
    .fetch_all(db.get_ref())
    .await
    .unwrap();

    let roles: Vec<String> = roles.into_iter().map(|record| record.name).collect();
    tracing::info!("User {} got roles {:?}", id, roles);

    Ok(roles)
}
