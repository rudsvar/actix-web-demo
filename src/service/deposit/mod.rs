//! Deposit related services.

use actix_web::web;

pub mod deposit_api;
pub mod deposit_model;
pub mod deposit_repository;

/// Configure the deposit service.
pub fn deposit_config(cfg: &mut web::ServiceConfig) {
    cfg.service(deposit_api::deposit);
}
