//! Models for representing deposits.

use serde::{Deserialize, Serialize};

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
