//! Functions for storing and retrieving subscriptions from a database.

use super::user_model::NewUser;
use crate::{
    error::DbError,
    middleware::security::Role,
    service::user::user_model::{HashedPassword, User},
};
use sqlx::PgExecutor;

/// Store a new user in the database.
pub async fn store_user(conn: impl PgExecutor<'_>, new_user: &NewUser) -> Result<User, DbError> {
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
    .fetch_one(conn)
    .await?;
    Ok(user)
}

/// Fetch a user from the database by id.
pub async fn fetch_user_by_id(conn: impl PgExecutor<'_>, id: &i32) -> Result<User, DbError> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, name, password as "password: HashedPassword", created_at FROM users WHERE id = $1"#,
        id
    )
    .fetch_one(conn)
    .await?;

    Ok(user)
}

/// Fetch a user from the database by name.
pub async fn fetch_user_by_username(
    conn: impl PgExecutor<'_>,
    username: &str,
) -> Result<User, DbError> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, name, password as "password: HashedPassword", created_at FROM users WHERE name = $1"#,
        username
    )
    .fetch_one(conn)
    .await?;

    Ok(user)
}

/// List all users.
pub async fn fetch_all_users(conn: impl PgExecutor<'_>) -> Result<Vec<User>, DbError> {
    let users = sqlx::query_as!(
        User,
        r#"SELECT id, name, password as "password: HashedPassword", created_at FROM users"#,
    )
    .fetch_all(conn)
    .await?;

    Ok(users)
}

/// Verify a password.
pub async fn authenticate(
    conn: impl PgExecutor<'_>,
    username: &str,
    password: &str,
) -> Result<Option<i32>, DbError> {
    let user = fetch_user_by_username(conn, username).await?;
    if user.password.verify(password) {
        Ok(Some(user.id))
    } else {
        Ok(None)
    }
}

/// Extract grants from the database.
pub async fn fetch_roles(conn: impl PgExecutor<'_>, username: &str) -> Result<Vec<Role>, DbError> {
    // Get roles from db
    let mut roles: Vec<Role> = sqlx::query!(
        r#"
        select r.name as "name: Role"
        from role r, user_role u_r, users u
        where r.id = u_r.role_id
          and u_r.user_id = u.id
          and u.name = $1;
        "#,
        username
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|r| r.name)
    .collect();

    // Add more roles based on role hierarchy
    let mut extra_roles = Vec::new();
    for r in &roles {
        if r == &Role::Admin {
            extra_roles.push(Role::User)
        }
    }
    roles.append(&mut extra_roles);

    tracing::info!("User {} got roles {:?}", username, roles);

    Ok(roles)
}
