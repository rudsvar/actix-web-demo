//! Functions for interacting with the request log.

use crate::infra::error::DbError;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use sqlx::PgExecutor;

#[derive(Builder)]
pub(crate) struct Request {
    #[builder(default, setter(strip_option))]
    user_id: Option<i32>,
    ip: String,
    request_method: String,
    request_uri: String,
    #[builder(default, setter(strip_option))]
    request_body: Option<String>,
    request_time: DateTime<Utc>,
    #[builder(default, setter(strip_option))]
    response_body: Option<String>,
    response_code: i32,
    response_time_ms: i32,
}

impl Request {}

/// Store a new request in the database.
pub(crate) async fn store_request(conn: impl PgExecutor<'_>, req: &Request) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        INSERT INTO requests (user_id, ip, request_method, request_uri, request_body, request_time, response_body, response_code, response_time_ms)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        req.user_id,
        req.ip,
        req.request_method,
        req.request_uri,
        req.request_body,
        req.request_time,
        req.response_body,
        req.response_code,
        req.response_time_ms
    )
    .execute(conn)
    .await?;

    Ok(())
}
