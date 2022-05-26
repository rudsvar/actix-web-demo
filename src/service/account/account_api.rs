//! An API for creating and modifying accounts.

use crate::error::{AppError, ServiceError};
use crate::security::jwt::{Claims, Role};
use crate::service::account::account_model::{Deposit, NewAccount, Withdrawal};
use crate::{error::DbError, DbPool};
use crate::{service::account::account_repository, AppResult};
use actix_web::{web, HttpResponse};
use actix_web_grants::proc_macro::has_roles;

/// Configures the account service.
pub fn account_config(cfg: &mut web::ServiceConfig) {
    cfg.service(post_account)
        .service(get_account)
        .service(deposit)
        .service(withdraw);
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

#[actix_web::post("/accounts/{id}/deposits")]
pub async fn deposit(
    db: web::Data<DbPool>,
    id: web::Path<i32>,
    deposit: web::Json<Deposit>,
) -> AppResult<HttpResponse> {
    let mut tx = db.begin().await.map_err(DbError::from)?;
    let account_id = id.into_inner();
    account_repository::deposit(&mut tx, account_id, deposit.amount()).await?;
    tx.commit().await.map_err(DbError::from)?;
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::post("/accounts/{id}/withdrawals")]
pub async fn withdraw(
    db: web::Data<DbPool>,
    id: web::Path<i32>,
    withdrawal: web::Json<Withdrawal>,
) -> AppResult<HttpResponse> {
    let mut tx = db.begin().await.map_err(DbError::from)?;

    let account_id = id.into_inner();
    let withdrawal = withdrawal.into_inner();

    // Try to make withdrawal
    tracing::debug!(
        "Withdrawing {} from account {}",
        withdrawal.amount(),
        account_id
    );
    let account = account_repository::withdraw(&mut tx, account_id, withdrawal.amount()).await?;

    // Check if balance became negative
    if account.balance() < 0 {
        return Err(ServiceError::ValidationError(format!(
            "Too low balance, required {} but had {}",
            withdrawal.amount(),
            account.balance() + withdrawal.amount() as i64
        ))
        .into());
    }

    tx.commit().await.map_err(DbError::from)?;
    Ok(HttpResponse::Ok().finish())
}
