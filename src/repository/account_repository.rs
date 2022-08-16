//! Utilities for interacting with the account table.

use crate::{
    infra::error::DbError,
    model::account_model::{Account, NewAccount},
    Tx,
};

/// Insert a new account into the account table.
#[tracing::instrument(skip(tx), ret)]
pub async fn insert_account(
    tx: &mut Tx,
    user_id: i32,
    new_account: NewAccount,
) -> Result<Account, DbError> {
    sqlx::query_as!(
        Account,
        r#"
            INSERT INTO accounts (name, balance, owner_id)
            VALUES ($1, $2, $3)
            RETURNING *
        "#,
        new_account.name(),
        0i64,
        user_id,
    )
    .fetch_one(tx)
    .await
    .map_err(DbError::from)
}

/// Fetch an account from the account table.
#[tracing::instrument(skip(tx), ret)]
pub async fn fetch_account(tx: &mut Tx, user_id: i32, account_id: i32) -> Result<Account, DbError> {
    sqlx::query_as!(
        Account,
        r#"SELECT * FROM accounts WHERE owner_id = $1 AND id = $2"#,
        user_id,
        account_id
    )
    .fetch_one(tx)
    .await
    .map_err(DbError::from)
}

/// Increase balance on an account.
#[tracing::instrument(skip(tx), ret)]
pub async fn deposit(tx: &mut Tx, account_id: i32, amount: u32) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        UPDATE accounts
        SET balance = balance + $1
        WHERE id = $2
        "#,
        amount as i64,
        account_id,
    )
    .execute(tx)
    .await?;
    Ok(())
}

/// Decrease balance on an account.
#[tracing::instrument(skip(tx), ret)]
pub async fn withdraw(tx: &mut Tx, account_id: i32, withdrawal: u32) -> Result<Account, DbError> {
    let account = sqlx::query_as!(
        Account,
        r#"
            UPDATE accounts
            SET balance = balance - $1
            WHERE id = $2
            RETURNING *
        "#,
        withdrawal as i64,
        account_id,
    )
    .fetch_one(tx)
    .await?;
    Ok(account)
}
