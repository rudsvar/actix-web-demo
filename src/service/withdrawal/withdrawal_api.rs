//! An API for withdrawing from an account.

use crate::{
    error::{BusinessError, DbError},
    service::{
        account::account_model::Account, withdrawal::withdrawal_model::Withdrawal, AppResult,
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
    let id = id.into_inner();

    // Check account balance
    let old_account = sqlx::query_as!(Account, "SELECT * FROM accounts WHERE id = $1", id)
        .fetch_one(&mut tx)
        .await
        .map_err(DbError::from)?;

    if old_account.balance < withdrawal.amount() as i64 {
        return Err(BusinessError::ValidationError(format!(
            "Balance is too low, required {} but had {}",
            old_account.balance(),
            withdrawal.amount()
        ))
        .into());
    }

    // Store in db
    sqlx::query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        withdrawal.amount() as i64,
        id
    )
    .execute(&mut tx)
    .await
    .map_err(DbError::from)?;

    tx.commit().await.map_err(DbError::from)?;

    Ok(HttpResponse::Created().finish())
}
