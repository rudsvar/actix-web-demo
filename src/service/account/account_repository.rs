//! Utilities for interacting with the account table.

use crate::error::DbError;
use sqlx::PgExecutor;

use super::account_model::{Account, NewAccount};

/// Insert a new account into the account table.
pub async fn insert_account(
    e: impl PgExecutor<'_>,
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
        new_account.owner_id()
    )
    .fetch_one(e)
    .await
    .map_err(DbError::from)
}

/// Fetch an account from the account table.
pub async fn fetch_account(e: impl PgExecutor<'_>, id: i32) -> Result<Account, DbError> {
    sqlx::query_as!(Account, r#"SELECT * FROM accounts WHERE id = $1"#, id)
        .fetch_one(e)
        .await
        .map_err(DbError::from)
}
