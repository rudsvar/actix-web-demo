//! Application wide errors.

use actix_http::{body::BoxBody, StatusCode};
use actix_web::ResponseError;
use config::ConfigError;
use thiserror::Error;

/// A general application error.
#[derive(Debug, Error)]
pub enum AppError {
    /// A logical error.
    #[error("{0}")]
    BusinessError(#[from] BusinessError),
    /// An external dependency failed.
    #[error("database error: {0}")]
    DbError(#[from] DbError),
    /// Could not load configuration.
    #[error("config error: {0}")]
    ConfigError(#[from] ConfigError),
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_http::StatusCode {
        tracing::error!("{}", self);
        match self {
            AppError::BusinessError(error) => error.status_code(),
            AppError::DbError(error) => error.status_code(),
            AppError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
    /// Error during authentication.
    #[error("authentication error")]
    AuthenticationError,
}

impl ResponseError for BusinessError {
    fn status_code(&self) -> StatusCode {
        match self {
            BusinessError::ValidationError(_) => StatusCode::BAD_REQUEST,
            BusinessError::AuthenticationError => StatusCode::UNAUTHORIZED,
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
