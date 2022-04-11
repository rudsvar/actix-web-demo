//! Application wide errors.

use actix_http::StatusCode;
use actix_web::ResponseError;
use thiserror::Error;

/// A general application error.
#[derive(Debug, Error)]
pub enum ApplicationError {
    /// An external dependency failed.
    #[error("database error: {0}")]
    DbError(#[from] DbError),
}

impl ResponseError for ApplicationError {
    fn status_code(&self) -> actix_http::StatusCode {
        tracing::error!("{}", self);
        match self {
            ApplicationError::DbError(error) => error.status_code(),
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
            _ => DbError::ConnectionError,
        }
    }
}
