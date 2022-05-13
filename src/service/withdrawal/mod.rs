//! Withdrawal related services.

use actix_web::web;

pub mod withdrawal_api;
pub mod withdrawal_model;
pub mod withdrawal_repository;

/// Configure the withdrawal service.
pub fn withdrawal_config(cfg: &mut web::ServiceConfig) {
    cfg.service(withdrawal_api::withdraw);
}
