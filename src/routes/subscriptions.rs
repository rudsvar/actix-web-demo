use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, Eq, Getters)]
pub struct NewFormData {
    email: String,
    name: String,
}

impl NewFormData {
    pub fn new(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, sqlx::FromRow, Getters)]
pub struct FormData {
    id: Uuid,
    email: String,
    name: String,
    subscribed_at: DateTime<Utc>,
}

impl FormData {
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

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name= %form.name
        )
    )]
#[actix_web::post("/subscriptions")]
pub async fn post_subscription(
    pool: web::Data<PgPool>,
    form: web::Form<NewFormData>,
) -> HttpResponse {
    let insert_result = insert_subscription(pool.get_ref(), &form).await;
    match insert_result {
        Ok(data) => HttpResponse::Created().json(data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::put("/subscriptions/{id}")]
pub async fn put_subscription(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    form: web::Form<NewFormData>,
) -> HttpResponse {
    let update_result = update_subscription(pool.get_ref(), &id, &form).await;
    match update_result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::get("/subscriptions")]
pub async fn list_subscriptions(pool: web::Data<PgPool>) -> HttpResponse {
    let query_result = fetch_all_subscriptions(pool.get_ref()).await;
    match query_result {
        Ok(posts) => HttpResponse::Ok().json(posts),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Inserts a subscription into the database.
#[tracing::instrument(name = "Saving new subscriber to the database", skip(pool, form))]
pub async fn insert_subscription(
    pool: &PgPool,
    form: &NewFormData,
) -> Result<FormData, sqlx::Error> {
    sqlx::query_as!(
        FormData,
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
    form: &NewFormData,
) -> Result<FormData, sqlx::Error> {
    sqlx::query_as!(
        FormData,
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
pub async fn fetch_all_subscriptions(pool: &PgPool) -> Result<Vec<FormData>, sqlx::Error> {
    sqlx::query_as!(FormData, r#"SELECT * FROM subscriptions"#)
        .fetch_all(pool)
        .await
}
