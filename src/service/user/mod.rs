//! User related services.

use actix_web::web;

pub mod user_api;
pub mod user_db;
pub mod user_model;

/// Configure the user service.
pub fn user_config(cfg: &mut web::ServiceConfig) {
    cfg.service(user_api::post_user)
        .service(user_api::get_user)
        .service(user_api::list_users);
}
