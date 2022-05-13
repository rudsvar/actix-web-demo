//! Models for representing withdrawals.

use serde::{Deserialize, Serialize};

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

    /// Get the withdrawal's amount.
    #[must_use]
    pub fn amount(&self) -> i32 {
        self.amount
    }
}
