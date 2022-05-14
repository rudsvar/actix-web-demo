//! An API for making deposits to an account.

use crate::{
    service::{
        deposit::{deposit_model::Deposit, deposit_repository},
        AppResult,
    },
    DbPool,
};
use actix_web::{web, HttpResponse};

#[actix_web::post("/accounts/{id}/deposits")]
pub async fn deposit(
    db: web::Data<DbPool>,
    id: web::Path<i32>,
    deposit: web::Json<Deposit>,
) -> AppResult<HttpResponse> {
    let account_id = id.into_inner();
    deposit_repository::deposit_into_account(db.get_ref(), account_id, deposit.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}