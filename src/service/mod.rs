//! Service implementations.
//!
//! Each service will generally include models,
//! integration with external services or the database,
//! and the API implementations themselves.

use crate::error::AppError;
use actix_web::HttpResponse;

pub mod account;
pub mod auth;
pub mod client_context;
pub mod health_check;
pub mod user;

/// A common response type for services.
pub type AppResponse = Result<HttpResponse, AppError>;
