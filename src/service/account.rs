//! An API for creating and modifying accounts.

use crate::security::Role;
use crate::{
    error::{BusinessError, DbError},
    security::AuthenticatedUser,
    service::AppResult,
    DbPool,
};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};
use actix_web_grants::{permissions::AuthDetails, proc_macro::has_roles};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A new account.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewAccount {
    name: String,
    owner_id: i32,
}

impl NewAccount {
    /// Creates a new account.
    #[must_use]
    pub fn new(name: String, owner_id: i32) -> Self {
        Self { name, owner_id }
    }
}

/// An existing account.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Account {
    /// The account id.
    pub id: i32,
    /// The name of the account.
    pub name: String,
    /// The current balance of the account.
    pub balance: i64,
    /// The owner of the account.
    pub owner_id: i32,
}

impl Account {
    /// Creates a new account.
    #[must_use]
    pub fn new(id: i32, name: String, balance: i64, owner_id: i32) -> Self {
        Self {
            id,
            name,
            balance,
            owner_id,
        }
    }

    /// Get the account's id.
    #[must_use]
    pub fn id(&self) -> i32 {
        self.id
    }

    /// Get a reference to the account's name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get the account's balance.
    #[must_use]
    pub fn balance(&self) -> i64 {
        self.balance
    }

    /// Get the account's owner id.
    #[must_use]
    pub fn owner_id(&self) -> i32 {
        self.owner_id
    }
}

#[actix_web::post("/accounts")]
pub async fn post_account(
    db: Data<DbPool>,
    new_account: Json<NewAccount>,
) -> AppResult<HttpResponse> {
    // Store in db
    let account = sqlx::query_as!(
        Account,
        r#"
        INSERT INTO accounts (name, balance, owner_id)
        VALUES ($1, $2, $3)
        RETURNING *"#,
        new_account.name,
        0i64,
        new_account.owner_id
    )
    .fetch_one(db.get_ref())
    .await
    .map_err(DbError::from)?;
    // Respond with newly created object
    Ok(HttpResponse::Created().json(account))
}

#[actix_web::get("/users/{user_id}/accounts/{account_id}")]
#[has_roles(
    "Role::User",
    type = "Role",
    secure = "path_params.0 == user.id() || roles.has_role(&Role::Admin)"
)]
pub async fn get_account(
    db: Data<DbPool>,
    user: AuthenticatedUser,
    roles: AuthDetails<Role>,
    path_params: Path<(i32, i32)>,
) -> AppResult<HttpResponse> {
    let (user_id, account_id) = path_params.into_inner();
    tracing::debug!("Getting account");
    let account = sqlx::query_as!(
        Account,
        r#"SELECT * FROM accounts WHERE id = $1 AND owner_id = $2"#,
        account_id,
        user_id,
    )
    .fetch_one(db.get_ref())
    .await
    .map_err(DbError::from)?;

    // Respond with newly created object or error message
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
    db: Data<DbPool>,
    id: Path<i32>,
    deposit: Json<Deposit>,
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
    db: Data<DbPool>,
    id: Path<i32>,
    withdrawal: Json<Withdrawal>,
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
    db: Data<DbPool>,
    new_transfer: Json<NewTransfer>,
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
