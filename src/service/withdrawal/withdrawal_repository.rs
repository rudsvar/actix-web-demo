//! Utilities for interacting with the withdrawal table.

use super::withdrawal_model::Withdrawal;
use crate::{error::DbError, service::account::account_model::Account};
use sqlx::PgExecutor;

/// Decrease balance on an account.
pub async fn withdraw(
    e: impl PgExecutor<'_>,
    account_id: i32,
    withdrawal: Withdrawal,
) -> Result<Account, DbError> {
    let account = sqlx::query_as!(
        Account,
        r#"
            UPDATE accounts
            SET balance = balance - $1
            WHERE id = $2
            RETURNING *
        "#,
        withdrawal.amount() as i32,
        account_id,
    )
    .fetch_one(e)
    .await?;
    Ok(account)
}
