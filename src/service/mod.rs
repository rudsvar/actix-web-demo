//! Service implementations.
//!
//! Each service will generally include models,
//! integration with external services or the database,
//! and the API implementations themselves.

use crate::error::ServiceError;
use actix_web::HttpResponse;

pub mod account;
pub mod auth;
pub mod client_context;
pub mod health_check;
pub mod user;

/// A common response type for services.
pub type ServiceResponse = Result<HttpResponse, ServiceError>;
