//! An API for creating and modifying accounts.

use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

/// A new account.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewAccount {
    name: String,
}

impl NewAccount {
    /// Creates a new account.
    #[must_use]
    pub fn new(name: String) -> Self {
        Self { name }
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
    pub balance: i32,
}

impl Account {
    /// Creates a new account.
    #[must_use]
    pub fn new(id: i32, name: String, balance: i32) -> Self {
        Self { id, name, balance }
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
    pub fn balance(&self) -> i32 {
        self.balance
    }
}

#[actix_web::post("/accounts")]
pub async fn post_account(db: Data<PgPool>, new_account: Json<NewAccount>) -> HttpResponse {
    // Store in db
    let account = sqlx::query_as!(
        Account,
        r#"INSERT INTO accounts (name, balance) VALUES ($1, $2) RETURNING *"#,
        new_account.name,
        0i32
    )
    .fetch_one(db.get_ref())
    .await
    .unwrap();
    // Respond with newly created object
    HttpResponse::Created().json(account)
}

#[actix_web::get("/accounts/{id}")]
pub async fn get_account(db: Data<PgPool>, id: Path<i32>) -> HttpResponse {
    let account = sqlx::query_as!(
        Account,
        r#"SELECT * FROM accounts WHERE id = $1"#,
        id.into_inner()
    )
    .fetch_one(db.get_ref())
    .await;

    // Respond with newly created object or error message
    match account {
        Ok(account) => HttpResponse::Ok().json(account),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// A deposit.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deposit {
    amount: i32,
}

impl Deposit {
    /// Creates a new deposit.
    #[must_use]
    pub fn new(amount: i32) -> Self {
        Self { amount }
    }
}

#[actix_web::post("/accounts/{id}/deposits")]
pub async fn deposit(db: Data<PgPool>, id: Path<i32>, deposit: Json<Deposit>) -> HttpResponse {
    let mut tx = db.begin().await.unwrap();
    let id = id.into_inner();

    // Store transaction
    let transaction = sqlx::query_as!(
        Deposit,
        "INSERT INTO transactions (account, amount) VALUES ($1, $2) returning amount",
        id,
        deposit.amount
    )
    .fetch_one(&mut tx)
    .await
    .unwrap();

    // Update account
    sqlx::query!(
        r#"
        UPDATE accounts
        SET balance = balance + $1
        WHERE id = $2
        "#,
        deposit.amount,
        id
    )
    .execute(&mut tx)
    .await
    .unwrap();

    tx.commit().await.unwrap();

    // Create response
    HttpResponse::Created().json(transaction)
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
    db: Data<PgPool>,
    id: Path<i32>,
    withdrawal: Json<Withdrawal>,
) -> HttpResponse {
    let mut tx = db.get_ref().begin().await.unwrap();
    let id = id.into_inner();

    // Store transaction
    let transaction = sqlx::query_as!(
        Withdrawal,
        "INSERT INTO transactions (account, amount) VALUES ($1, $2) RETURNING amount",
        id,
        -withdrawal.amount
    )
    .fetch_one(&mut tx)
    .await
    .unwrap();

    // Check account balance
    let old_account = sqlx::query_as!(Account, "SELECT * FROM accounts WHERE id = $1", id)
        .fetch_one(&mut tx)
        .await
        .unwrap();

    if old_account.balance < withdrawal.amount {
        return HttpResponse::BadRequest().json(format!(
            "Balance is too low, had {} but required {}",
            old_account.balance, withdrawal.amount
        ));
    }

    // Store in db
    sqlx::query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        withdrawal.amount,
        id
    )
    .execute(&mut tx)
    .await
    .unwrap();

    tx.commit().await.unwrap();

    HttpResponse::Created().json(transaction)
}

/// A new transfer between accounts.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NewTransfer {
    /// The account to take money from.
    pub from_account: i32,
    /// The account to send money to.
    pub to_account: i32,
    /// The amount of money.
    pub amount: i32,
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
    pub amount: i32,
    /// A timestamp for the transaction.
    pub created_at: DateTime<Utc>,
}

#[actix_web::post("/transfers")]
pub async fn transfer(db: Data<PgPool>, new_transfer: Json<NewTransfer>) -> HttpResponse {
    let mut tx = db.get_ref().begin().await.unwrap();

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
        new_transfer.amount,
    )
    .fetch_one(&mut tx)
    .await
    .unwrap();

    // Verify old account
    let old_account = sqlx::query_as!(
        Account,
        "SELECT * FROM accounts WHERE id = $1",
        transfer.from_account
    )
    .fetch_one(&mut tx)
    .await
    .unwrap();

    if old_account.balance < transfer.amount {
        return HttpResponse::BadRequest().json(format!(
            "Balance is too low, had {} but required {}",
            old_account.balance, transfer.amount
        ));
    }

    // Take money from account
    sqlx::query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        transfer.amount,
        transfer.from_account
    )
    .execute(&mut tx)
    .await
    .unwrap();

    // Take money from account
    sqlx::query!(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        transfer.amount,
        transfer.to_account
    )
    .execute(&mut tx)
    .await
    .unwrap();

    tx.commit().await.unwrap();

    HttpResponse::Created().json(transfer)
}
