//! Utilities for interacting with the withdrawal table.

use super::withdrawal_model::Withdrawal;
use crate::error::DbError;
use sqlx::PgExecutor;

/// Decrease balance on an account.
pub async fn withdraw_from_account(
    e: impl PgExecutor<'_>,
    account_id: i32,
    withdrawal: Withdrawal,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        UPDATE accounts
        SET balance = balance - $1
        WHERE id = $2
        "#,
        withdrawal.amount() as i32,
        account_id,
    )
    .execute(e)
    .await?;
    Ok(())
}
