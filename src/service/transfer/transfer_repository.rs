//! Utilities for interacting with the account table.

use sqlx::{Postgres, Transaction};

use crate::{
    error::{AppError, BusinessError, DbError},
    service::account::account_model::Account,
};

use super::transfer_model::{NewTransfer, Transfer};

/// Performs a transfer between two accounts.
pub async fn insert_transfer(
    tx: &mut Transaction<'_, Postgres>,
    new_transfer: NewTransfer,
) -> Result<Transfer, AppError> {
    // Verify old account
    let old_account = sqlx::query_as!(
        Account,
        "SELECT * FROM accounts WHERE id = $1",
        new_transfer.from_account
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(DbError::from)?;

    if new_transfer.amount as i64 > old_account.balance {
        return Err(BusinessError::ValidationError(format!(
            "Balance is too low, required {} but had {}",
            new_transfer.amount, old_account.balance
        ))
        .into());
    }

    // Take money from account
    sqlx::query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        new_transfer.amount as i64,
        new_transfer.from_account
    )
    .execute(&mut *tx)
    .await
    .map_err(DbError::from)?;

    // Give money to other account
    sqlx::query!(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        new_transfer.amount as i64,
        new_transfer.to_account
    )
    .execute(&mut *tx)
    .await
    .map_err(DbError::from)?;

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
    .fetch_one(tx)
    .await
    .map_err(DbError::from)?;

    Ok(transfer)
}
