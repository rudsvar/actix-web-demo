//! Service implementations.
//!
//! Each service will generally include models,
//! integration with external services or the database,
//! and the API implementations themselves.

use crate::error::AppError;

pub mod account;
pub mod client_context;
pub mod health_check;
pub mod token;
pub mod transfer;
pub mod user;

/// A common response type for services.
pub type AppResult<T> = Result<T, AppError>;
