//! Routes for user management.

use crate::model::user_model::NewUser;
use crate::repository::user_repository;
use crate::security::jwt::{Claims, Role};
use crate::{infra::error::AppError, DbPool};
use actix_web::{
    web::{self, Data, Json, Path, ReqData},
    HttpResponse,
};
use actix_web_grants::proc_macro::has_roles;

/// Configure the user service.
pub fn user_config(cfg: &mut web::ServiceConfig) {
    cfg.service(post_user).service(get_user).service(list_users);
}

#[actix_web::post("/users")]
#[has_roles("Role::Admin", type = "Role")]
pub async fn post_user(
    db: Data<DbPool>,
    new_user: Json<NewUser>,
) -> Result<HttpResponse, AppError> {
    let new_user = new_user.into_inner();
    let user = user_repository::store_user(db.get_ref(), &new_user).await?;
    Ok(HttpResponse::Created().json(user))
}

#[actix_web::get("/users/{id}")]
#[has_roles(
    "Role::Admin",
    type = "Role",
    secure = "*id == claims.id() || claims.has_role(&Role::Admin)"
)]
pub async fn get_user(
    db: Data<DbPool>,
    claims: ReqData<Claims>,
    id: Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user = user_repository::fetch_user_by_id(db.get_ref(), &id).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[actix_web::get("/users")]
#[has_roles("Role::Admin", type = "Role")]
pub async fn list_users(db: Data<DbPool>) -> Result<HttpResponse, AppError> {
    let users = user_repository::fetch_all_users(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(users))
}
