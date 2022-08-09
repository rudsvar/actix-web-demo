//! Models representing transfers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
