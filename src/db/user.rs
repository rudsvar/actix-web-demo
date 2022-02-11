//! Functions for storing and retrieving subscriptions from a database.

use crate::model::user::{HashedPassword, User};
use sqlx::{postgres::PgQueryResult, PgPool};
use uuid::Uuid;

/// Store a new user in the database.
pub async fn store_user(pool: &PgPool, new_user: &User) -> Result<PgQueryResult, sqlx::Error> {
    tracing::info!("Storing user with uuid {}", &new_user.id);
    sqlx::query!(
        "INSERT INTO users VALUES ($1, $2, $3, $4)",
        &new_user.id,
        &new_user.name,
        &new_user.password.hashed_password(),
        &new_user.created_at,
    )
    .execute(pool)
    .await
}

/// Fetch a user from the database by id.
pub async fn fetch_user_by_id(pool: &PgPool, id: &Uuid) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"SELECT id, name, password as "password: HashedPassword", created_at FROM users WHERE id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
}

/// List all users.
pub async fn fetch_all_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"SELECT id, name, password as "password: HashedPassword", created_at FROM users"#,
    )
    .fetch_all(pool)
    .await
}
