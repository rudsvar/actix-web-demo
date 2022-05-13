//! An API for creating and modifying accounts.

use crate::security::{Claims, Role};
use crate::service::account::account_model::{Account, NewAccount};
use crate::service::account::account_repository;
use crate::{
    error::{BusinessError, DbError},
    service::AppResult,
    DbPool,
};
use actix_web::{web, HttpResponse};
use actix_web_grants::proc_macro::has_roles;
use serde::{Deserialize, Serialize};

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
    tx.commit().await.map_err(DbError::from)?;
    Ok(HttpResponse::Ok().json(account))
}

/// A deposit.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deposit {
    amount: u32,
}

impl Deposit {
    /// Creates a new deposit.
    #[must_use]
    pub fn new(amount: u32) -> Self {
        Self { amount }
    }
}

#[actix_web::post("/accounts/{id}/deposits")]
pub async fn deposit(
    db: web::Data<DbPool>,
    id: web::Path<i32>,
    deposit: web::Json<Deposit>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();

    if deposit.amount >= i32::MAX as u32 {
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
        deposit.amount as i32,
        id
    )
    .execute(db.get_ref())
    .await
    .map_err(DbError::from)?;

    // Create response
    Ok(HttpResponse::Created().finish())
}

/// A withdrawal.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Withdrawal {
    amount: i32,
}

impl Withdrawal {
    /// Creates a new withdrawal.
    #[must_use]
    pub fn new(amount: i32) -> Self {
        Self { amount }
    }
}

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

    if old_account.balance < withdrawal.amount as i64 {
        return Ok(HttpResponse::BadRequest().json(format!(
            "Balance is too low, had {} but required {}",
            old_account.balance, withdrawal.amount
        )));
    }

    // Store in db
    sqlx::query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        withdrawal.amount as i64,
        id
    )
    .execute(&mut tx)
    .await
    .map_err(DbError::from)?;

    tx.commit().await.map_err(DbError::from)?;

    Ok(HttpResponse::Created().finish())
}
