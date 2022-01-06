//! Functions for storing and retrieving subscriptions from a database.

use crate::model::subscription::{NewSubscription, Subscription};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

/// Inserts a subscription into the database.
#[tracing::instrument(skip(pool, form))]
pub async fn insert_subscription(
    pool: &PgPool,
    form: &NewSubscription,
) -> Result<Subscription, sqlx::Error> {
    sqlx::query_as!(
        Subscription,
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, name, subscribed_at
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {}", e);
        e
    })
}

/// Updates a subscription in the database.
pub async fn update_subscription(
    pool: &PgPool,
    id: &Uuid,
    form: &NewSubscription,
) -> Result<Subscription, sqlx::Error> {
    sqlx::query_as!(
        Subscription,
        r#"
        UPDATE subscriptions SET email = $1, name = $2
        WHERE id = $3
        RETURNING id, email, name, subscribed_at
        "#,
        form.email,
        form.name,
        id
    )
    .fetch_one(pool)
    .await
}

/// Fetches all subscriptions from the database.
pub async fn fetch_all_subscriptions(pool: &PgPool) -> Result<Vec<Subscription>, sqlx::Error> {
    sqlx::query_as!(Subscription, r#"SELECT * FROM subscriptions"#)
        .fetch_all(pool)
        .await
}
