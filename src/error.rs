//! Application wide errors.

use actix_http::{body::BoxBody, StatusCode};
use actix_web::ResponseError;
use config::ConfigError;
use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

/// A general application error.
#[derive(Debug, Error)]
pub enum AppError {
    /// A logical error.
    #[error("{0}")]
    BusinessError(#[from] ServiceError),
    /// An external dependency failed.
    #[error("database error: {0}")]
    DbError(#[from] DbError),
    /// Could not load configuration.
    #[error("config error: {0}")]
    ConfigError(#[from] ConfigError),
    /// Error during authentication.
    #[error("authentication error")]
    AuthenticationError,
    /// Error during authentication.
    #[error("authorization error")]
    AuthorizationError,
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_http::StatusCode {
        tracing::error!("{}", self);
        match self {
            AppError::BusinessError(error) => error.status_code(),
            AppError::DbError(error) => error.status_code(),
            AppError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AuthenticationError => StatusCode::UNAUTHORIZED,
            AppError::AuthorizationError => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<BoxBody> {
        let res = actix_web::HttpResponse::new(self.status_code());
        res.set_body(BoxBody::new(format!("{}", &self)))
    }
}

/// A logical error for when the operation could not be performed.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// A validation failed.
    #[error("{0}")]
    ValidationError(String),
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::ValidationError(_) => StatusCode::BAD_REQUEST,
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
    /// Connection error.
    #[error("postgres error: {0}")]
    PgDatabaseError(Box<PgDatabaseError>),
    /// Other error.
    #[error("{0}")]
    Other(sqlx::Error),
}

impl ResponseError for DbError {
    fn status_code(&self) -> actix_http::StatusCode {
        match self {
            DbError::NotFound => StatusCode::NOT_FOUND,
            DbError::Conflict => StatusCode::CONFLICT,
            DbError::ConnectionError => StatusCode::INTERNAL_SERVER_ERROR,
            DbError::PgDatabaseError(e) => match e.code() {
                "23514" => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            DbError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sqlx::Error> for DbError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => DbError::NotFound,
            sqlx::Error::Io(_) => DbError::ConnectionError,
            sqlx::Error::Database(e) => {
                // Check if PostgreSQL error
                let pg_error = e.try_downcast::<PgDatabaseError>();
                match pg_error {
                    Ok(pg_error) => DbError::PgDatabaseError(pg_error),
                    Err(e) => DbError::Other(sqlx::Error::Database(e)),
                }
            }
            e => DbError::Other(e),
        }
    }
}
