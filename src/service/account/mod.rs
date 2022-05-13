//! Account related services.

use actix_web::web;

pub mod account_api;
pub mod account_model;
pub mod account_repository;

/// Configure the account service.
pub fn account_config(cfg: &mut web::ServiceConfig) {
    cfg.service(account_api::post_account)
        .service(account_api::get_account);
}
