//! An API for transferring money between accounts.

use crate::{
    error::{DbError, ServiceError},
    security::jwt::{Claims, Role},
    service::{
        account::account_repository,
        transfer::{transfer_model::NewTransfer, transfer_repository},
    },
    AppResult, DbPool,
};
use actix_web::{web, HttpResponse};
use actix_web_grants::proc_macro::has_roles;

/// Configure the transfer service.
pub fn transfer_config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_transfer);
}

#[actix_web::post("/transfers")]
#[has_roles(
    "Role::User",
    type = "Role",
    secure = "new_transfer.from_account == claims.id() || claims.has_role(&Role::Admin)"
)]
pub async fn create_transfer(
    db: web::Data<DbPool>,
    claims: web::ReqData<Claims>,
    new_transfer: web::Json<NewTransfer>,
) -> AppResult<HttpResponse> {
    let mut tx = db.get_ref().begin().await.map_err(DbError::from)?;

    // Verify old account
    let old_account = account_repository::fetch_account(&mut tx, new_transfer.from_account).await?;

    if new_transfer.amount as i64 > old_account.balance() {
        return Err(ServiceError::ValidationError(format!(
            "Balance is too low, required {} but had {}",
            new_transfer.amount,
            old_account.balance()
        ))
        .into());
    }

    // Take from account
    account_repository::withdraw(&mut tx, new_transfer.from_account, new_transfer.amount).await?;

    // Give to account
    account_repository::deposit(&mut tx, new_transfer.to_account, new_transfer.amount).await?;

    // Insert transfer
    let transfer = transfer_repository::insert_transfer(&mut tx, new_transfer.into_inner()).await?;

    tx.commit().await.map_err(DbError::from)?;

    Ok(HttpResponse::Created().json(transfer))
}
