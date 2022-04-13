//! Functions for storing and retrieving subscriptions from a database.

use super::user_model::NewUser;
use crate::{
    error::DbError,
    service::user::user_model::{HashedPassword, User},
    DbPool,
};

/// Store a new user in the database.
pub async fn store_user(pool: &DbPool, new_user: &NewUser) -> Result<User, DbError> {
    tracing::info!("Storing user with name {}", &new_user.name);
    let hashed_password = HashedPassword::new(new_user.password.clone());
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (name, password)
        VALUES ($1, $2)
        RETURNING id, name, password as "password: HashedPassword", created_at
        "#,
        &new_user.name,
        &hashed_password.hashed_password()
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

/// Fetch a user from the database by id.
pub async fn fetch_user_by_id(pool: &DbPool, id: &i32) -> Result<User, DbError> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, name, password as "password: HashedPassword", created_at FROM users WHERE id = $1"#,
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// List all users.
pub async fn fetch_all_users(pool: &DbPool) -> Result<Vec<User>, DbError> {
    let users = sqlx::query_as!(
        User,
        r#"SELECT id, name, password as "password: HashedPassword", created_at FROM users"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}

/// Verify a password.
pub async fn verify_password(pool: &DbPool, id: &i32, password: &str) -> Result<bool, DbError> {
    let user = fetch_user_by_id(pool, id).await?;
    Ok(user.password.verify(password))
}