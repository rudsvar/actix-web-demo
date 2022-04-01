//! An API for creating and modifying accounts.

use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};
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
pub async fn deposit(db: Data<PgPool>, deposit: Json<Deposit>) -> HttpResponse {
    // Store in db
    let account = sqlx::query_as!(
        Account,
        r#"UPDATE accounts SET balance = balance + $1 RETURNING *"#,
        deposit.amount,
    )
    .fetch_one(db.get_ref())
    .await
    .unwrap();
    // Create response
    HttpResponse::Created().json(account)
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
    let mut transaction = db.get_ref().begin().await.unwrap();
    let id = id.into_inner();

    // Check account balance
    let old_account = sqlx::query_as!(Account, "SELECT * FROM accounts WHERE id = $1", id)
        .fetch_one(&mut transaction)
        .await
        .unwrap();

    if old_account.balance < withdrawal.amount {
        return HttpResponse::BadRequest().json(format!(
            "Balance is too low, had {} but required {}",
            old_account.balance, withdrawal.amount
        ));
    }

    // Store in db
    let account = sqlx::query_as!(
        Account,
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2 RETURNING *",
        withdrawal.amount,
        id
    )
    .fetch_one(&mut transaction)
    .await
    .unwrap();

    transaction.commit().await.unwrap();

    HttpResponse::Created().json(account)
}
