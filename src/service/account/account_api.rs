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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
    let account = account_repository::insert(&mut tx, new_account.into_inner()).await?;
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
    let account = account_repository::fetch(&mut tx, account_id).await?;
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

fn validate<T, P>(t: &T, p: P, msg: impl Into<String>) -> Result<(), BusinessError>
where
    P: Fn(&T) -> bool,
{
    if p(t) {
        Ok(())
    } else {
        Err(BusinessError::ValidationError(msg.into()))
    }
}

#[actix_web::post("/accounts/{id}/deposits")]
pub async fn deposit(
    db: web::Data<DbPool>,
    id: web::Path<i32>,
    deposit: web::Json<Deposit>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();

    validate(
        &deposit.amount,
        |v| *v <= i32::MAX as u32,
        format!("amount must be lower than {}", i32::MAX),
    )?;

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

/// A new transfer between accounts.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NewTransfer {
    /// The account to take money from.
    pub from_account: i32,
    /// The account to send money to.
    pub to_account: i32,
    /// The amount of money.
    pub amount: u32,
}

/// A stored transfer between accounts.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Transfer {
    /// The id of the transfer.
    pub id: i32,
    /// The account to take money from.
    pub from_account: i32,
    /// The account to send money to.
    pub to_account: i32,
    /// The amount of money.
    pub amount: i64,
    /// A timestamp for the transaction.
    pub created_at: DateTime<Utc>,
}

#[actix_web::post("/transfers")]
pub async fn transfer(
    db: web::Data<DbPool>,
    new_transfer: web::Json<NewTransfer>,
) -> AppResult<HttpResponse> {
    let mut tx = db.get_ref().begin().await.map_err(DbError::from)?;

    // Store transaction
    let transfer = sqlx::query_as!(
        Transfer,
        r#"
        INSERT INTO transfers (from_account, to_account, amount)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        new_transfer.from_account,
        new_transfer.to_account,
        new_transfer.amount as i64,
    )
    .fetch_one(&mut tx)
    .await
    .map_err(DbError::from)?;

    // Verify old account
    let old_account = sqlx::query_as!(
        Account,
        "SELECT * FROM accounts WHERE id = $1",
        transfer.from_account
    )
    .fetch_one(&mut tx)
    .await
    .map_err(DbError::from)?;

    validate(
        &transfer.amount,
        |&amount| amount < old_account.balance,
        format!(
            "Balance is too low, had {} but required {}",
            old_account.balance, transfer.amount
        ),
    )?;

    // Take money from account
    sqlx::query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        transfer.amount,
        transfer.from_account
    )
    .execute(&mut tx)
    .await
    .map_err(DbError::from)?;

    // Take money from account
    sqlx::query!(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        transfer.amount,
        transfer.to_account
    )
    .execute(&mut tx)
    .await
    .map_err(DbError::from)?;

    tx.commit().await.map_err(DbError::from)?;

    Ok(HttpResponse::Created().json(transfer))
}
