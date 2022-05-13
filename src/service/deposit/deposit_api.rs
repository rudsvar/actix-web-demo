//! An API for making deposits to an account.

use crate::{
    error::{BusinessError, DbError},
    service::{deposit::deposit_model::Deposit, AppResult},
    DbPool,
};
use actix_web::{web, HttpResponse};

#[actix_web::post("/accounts/{id}/deposits")]
pub async fn deposit(
    db: web::Data<DbPool>,
    id: web::Path<i32>,
    deposit: web::Json<Deposit>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();

    if deposit.amount() >= i32::MAX as u32 {
        return Err(BusinessError::ValidationError(format!(
            "Deposit amount must be less than or equal to {}",
            i32::MAX
        ))
        .into());
    }

    // Update account
    sqlx::query!(
        r#"
        UPDATE accounts
        SET balance = balance + $1
        WHERE id = $2
        "#,
        deposit.amount() as i32,
        id
    )
    .execute(db.get_ref())
    .await
    .map_err(DbError::from)?;

    // Create response
    Ok(HttpResponse::Created().finish())
}
