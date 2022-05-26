//! An API for creating and modifying accounts.

use crate::error::AppError;
use crate::security::jwt::{Claims, Role};
use crate::service::account::account_model::NewAccount;
use crate::service::account::account_repository;
use crate::AppResult;
use crate::{error::DbError, DbPool};
use actix_web::{web, HttpResponse};
use actix_web_grants::proc_macro::has_roles;

/// Configures the account service.
pub fn account_config(cfg: &mut web::ServiceConfig) {
    cfg.service(post_account).service(get_account);
}

#[actix_web::post("/accounts")]
#[has_roles(
    "Role::User",
    type = "Role",
    secure = "new_account.owner_id() == claims.id() || claims.has_role(&Role::Admin)"
)]
pub async fn post_account(
    db: web::Data<DbPool>,
    claims: web::ReqData<Claims>,
    new_account: web::Json<NewAccount>,
) -> AppResult<HttpResponse> {
    let mut tx = db.begin().await.map_err(DbError::from)?;
    let account = account_repository::insert_account(&mut tx, new_account.into_inner()).await?;
    tx.commit().await.map_err(DbError::from)?;
    Ok(HttpResponse::Created().json(account))
}

#[actix_web::get("/users/{user_id}/accounts/{account_id}")]
#[has_roles(
    "Role::User",
    type = "Role",
    secure = "path_params.0 == claims.id() || claims.has_role(&Role::Admin)"
)]
pub async fn get_account(
    db: web::Data<DbPool>,
    claims: web::ReqData<Claims>,
    path_params: web::Path<(i32, i32)>,
) -> AppResult<HttpResponse> {
    let account_id = path_params.1;
    let mut tx = db.begin().await.map_err(DbError::from)?;
    let account = account_repository::fetch_account(&mut tx, account_id).await?;
    if account.owner_id != claims.id() && !claims.has_role(&Role::Admin) {
        return Err(AppError::AuthorizationError);
    }
    tx.commit().await.map_err(DbError::from)?;
    Ok(HttpResponse::Ok().json(account))
}
