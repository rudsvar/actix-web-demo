//! Account related types.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

    /// Get a reference to the new account's name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_ref()
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
    pub balance: i64,
    /// The owner of the account.
    pub owner_id: i32,
}

impl Account {
    /// Creates a new account.
    #[must_use]
    pub fn new(id: i32, name: String, balance: i64, owner_id: i32) -> Self {
        Self {
            id,
            name,
            balance,
            owner_id,
        }
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
    pub fn balance(&self) -> i64 {
        self.balance
    }

    /// Get the account's owner id.
    #[must_use]
    pub fn owner_id(&self) -> i32 {
        self.owner_id
    }
}

/// A deposit.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deposit {
    amount: u32,
}

impl Deposit {
    /// Creates a new deposit.
    #[must_use]
    pub fn new(amount: u32) -> Self {
        Self { amount }
    }

    /// Get the deposit's amount.
    #[must_use]
    pub fn amount(&self) -> u32 {
        self.amount
    }
}

/// A withdrawal.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Withdrawal {
    amount: u32,
}

impl Withdrawal {
    /// Creates a new withdrawal.
    #[must_use]
    pub fn new(amount: u32) -> Self {
        Self { amount }
    }

    /// Get the withdrawal's amount.
    #[must_use]
    pub fn amount(&self) -> u32 {
        self.amount
    }
}
