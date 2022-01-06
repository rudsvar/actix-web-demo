use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct NewSubscription {
    pub email: String,
    pub name: String,
}

impl NewSubscription {
    pub fn new(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, sqlx::FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub subscribed_at: DateTime<Utc>,
}

impl Subscription {
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
