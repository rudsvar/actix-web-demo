//! A model for transfers between accounts.

use actix_web::web;

pub mod transfer_api;
pub mod transfer_model;
pub mod transfer_repository;

/// Configure the transfer service.
pub fn transfer_config(cfg: &mut web::ServiceConfig) {
    cfg.service(transfer_api::create_transfer);
}
