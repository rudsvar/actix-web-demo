//! Models for representing subscriptions.

use std::ops::Deref;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A wrapper type that guarantees that a password is hashed.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::FromRow, sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct HashedPassword(String);

impl HashedPassword {
    /// Creates a hashed password from an unhashed [`String`].
    pub fn new(password: String) -> Self {
        Self(bcrypt::hash(password, 10).unwrap())
    }
    /// Returns the hashed password.
    pub fn hashed_password(&self) -> &str {
        &self.0
    }
    /// Compares the provided string to the stored password.
    pub fn verify(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.0).unwrap()
    }
}

impl Deref for HashedPassword {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A new user.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct NewUser {
    /// The name of the new user.
    pub name: String,
    /// The password of the new user.
    pub password: String,
}

/// The more general user model.
#[derive(Debug, Serialize, PartialEq, Eq, sqlx::FromRow)]
pub struct User {
    /// The id.
    pub id: Uuid,
    /// The name of the new user.
    pub name: String,
    /// The password of the new user.
    pub password: HashedPassword,
    /// Creation date.
    pub created_at: DateTime<Utc>,
}
