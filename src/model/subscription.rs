//! Models for representing subscriptions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A new subscription, usually received from a user.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct NewSubscription {
    /// The email of the newly subscribed user.
    pub email: String,
    /// The name of the newly subscribed user.
    pub name: String,
}

impl NewSubscription {
    /// Constructs a new [`NewSubscription`].
    pub fn new(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: name.into(),
        }
    }
}

/// A subscription.
#[derive(Debug, Serialize, PartialEq, Eq, sqlx::FromRow)]
pub struct Subscription {
    /// The id of the subscription.
    pub id: Uuid,
    /// The email of the subscribed user.
    pub email: String,
    /// The name of the subscribed user.
    pub name: String,
    /// The time the user subscribed at.
    pub subscribed_at: DateTime<Utc>,
}

impl Subscription {
    /// Constructs a new [`Subscription`].
    pub fn new(
        id: Uuid,
        email: impl Into<String>,
        name: impl Into<String>,
        subscribed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email: email.into(),
            name: name.into(),
            subscribed_at,
        }
    }
}
