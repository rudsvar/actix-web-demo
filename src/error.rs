//! Application wide errors.

use actix_http::{body::BoxBody, StatusCode};
use actix_web::ResponseError;
use thiserror::Error;

/// A general application error.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// A logical error.
    #[error("business error: {0}")]
    BusinessError(#[from] BusinessError),
    /// An external dependency failed.
    #[error("database error: {0}")]
    DbError(#[from] DbError),
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> actix_http::StatusCode {
        tracing::error!("{}", self);
        match self {
            ServiceError::BusinessError(error) => error.status_code(),
            ServiceError::DbError(error) => error.status_code(),
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<BoxBody> {
        let res = actix_web::HttpResponse::new(self.status_code());
        res.set_body(BoxBody::new(format!("{}", &self)))
    }
}

/// A logical error for when the operation could not be performed.
#[derive(Debug, Error)]
pub enum BusinessError {
    /// A validation failed.
    #[error("{0}")]
    ValidationError(String),
}

impl ResponseError for BusinessError {
    fn status_code(&self) -> StatusCode {
        match self {
            BusinessError::ValidationError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

/// Error representing a failure at the database layer.
#[derive(Debug, Error)]
pub enum DbError {
    /// Not found.
    #[error("entity not found")]
    NotFound,
    /// Conflict.
    #[error("entity already exists")]
    Conflict,
    /// Connection error.
    #[error("could not connect to database")]
    ConnectionError,
    /// Other error.
    #[error("{0}")]
    Other(sqlx::Error),
}

impl ResponseError for DbError {
    fn status_code(&self) -> actix_http::StatusCode {
        tracing::error!("{}", self);
        match self {
            DbError::NotFound => StatusCode::NOT_FOUND,
            DbError::Conflict => StatusCode::CONFLICT,
            DbError::ConnectionError => StatusCode::INTERNAL_SERVER_ERROR,
            DbError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sqlx::Error> for DbError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => DbError::NotFound,
            sqlx::Error::Io(_) => DbError::ConnectionError,
            e => DbError::Other(e),
        }
    }
}
