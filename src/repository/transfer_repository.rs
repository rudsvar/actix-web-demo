//! Utilities for interacting with the account table.

use crate::{
    error::DbError,
    model::transfer_model::{NewTransfer, Transfer},
};
use sqlx::{Postgres, Transaction};

/// Performs a transfer between two accounts.
pub async fn insert_transfer(
    tx: &mut Transaction<'_, Postgres>,
    new_transfer: NewTransfer,
) -> Result<Transfer, DbError> {
    // Store transfer
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
    .await?;
    Ok(transfer)
}
