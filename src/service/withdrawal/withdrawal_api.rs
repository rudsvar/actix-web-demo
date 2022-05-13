//! An API for withdrawing from an account.

use crate::{
    error::{BusinessError, DbError},
    service::{
        account::account_repository,
        withdrawal::{withdrawal_model::Withdrawal, withdrawal_repository},
        AppResult,
    },
    DbPool,
};
use actix_web::{web, HttpResponse};

#[actix_web::post("/accounts/{id}/withdrawals")]
pub async fn withdraw(
    db: web::Data<DbPool>,
    id: web::Path<i32>,
    withdrawal: web::Json<Withdrawal>,
) -> AppResult<HttpResponse> {
    let mut tx = db.get_ref().begin().await.map_err(DbError::from)?;
    let account_id = id.into_inner();

    // Check current account balance
    let old_account = account_repository::fetch_account(&mut tx, account_id).await?;

    if old_account.balance < withdrawal.amount() as i64 {
        return Err(BusinessError::ValidationError(format!(
            "Balance is too low, required {} but had {}",
            old_account.balance(),
            withdrawal.amount()
        ))
        .into());
    }

    // Make withdrawal
    withdrawal_repository::withdraw_from_account(&mut tx, account_id, withdrawal.into_inner())
        .await?;
    tx.commit().await.map_err(DbError::from)?;
    Ok(HttpResponse::Ok().finish())
}
