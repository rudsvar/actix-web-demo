//! An API for transferring money between accounts.

use crate::{
    error::{ServiceError, DbError},
    service::{
        account::account_repository,
        deposit::{deposit_model::Deposit, deposit_repository},
        transfer::{transfer_model::NewTransfer, transfer_repository},
        withdrawal::{withdrawal_model::Withdrawal, withdrawal_repository},
        AppResult,
    },
    DbPool,
};
use actix_web::{web, HttpResponse};

#[actix_web::post("/transfers")]
pub async fn create_transfer(
    db: web::Data<DbPool>,
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
    withdrawal_repository::withdraw(
        &mut tx,
        new_transfer.from_account,
        Withdrawal::new(new_transfer.amount),
    )
    .await?;

    // Give to account
    deposit_repository::deposit_into_account(
        &mut tx,
        new_transfer.from_account,
        Deposit::new(new_transfer.amount),
    )
    .await?;

    // Insert transfer
    let transfer = transfer_repository::insert_transfer(&mut tx, new_transfer.into_inner()).await?;

    tx.commit().await.map_err(DbError::from)?;

    Ok(HttpResponse::Created().json(transfer))
}
