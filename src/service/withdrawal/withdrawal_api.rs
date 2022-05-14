//! An API for withdrawing from an account.

use crate::{
    error::{DbError, ServiceError},
    service::{
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
    let withdrawal = withdrawal.into_inner();

    // Try to make withdrawal
    tracing::debug!(
        "Withdrawing {} from account {}",
        withdrawal.amount(),
        account_id
    );
    let account = withdrawal_repository::withdraw(&mut tx, account_id, withdrawal).await?;

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
