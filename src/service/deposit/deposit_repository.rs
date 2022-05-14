//! Utilities for interacting with the deposit table.

use super::deposit_model::Deposit;
use crate::error::DbError;
use sqlx::PgExecutor;

/// Increase balance on an account.
pub async fn deposit(
    e: impl PgExecutor<'_>,
    account_id: i32,
    deposit: Deposit,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        UPDATE accounts
        SET balance = balance + $1
        WHERE id = $2
        "#,
        deposit.amount() as i32,
        account_id,
    )
    .execute(e)
    .await?;
    Ok(())
}
