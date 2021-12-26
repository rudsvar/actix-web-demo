use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, Eq)]
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
    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, sqlx::FromRow)]
pub struct FormData {
    id: Uuid,
    email: String,
    name: String,
    subscribed_at: DateTime<Utc>,
}

impl FormData {
    pub fn new(id: Uuid, email: String, name: String, subscribed_at: DateTime<Utc>) -> Self {
        Self {
            id,
            email,
            name,
            subscribed_at,
        }
    }
    pub fn id(&self) -> &Uuid {
        &self.id
    }
    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn subscribed_at(&self) -> &DateTime<Utc> {
        return &self.subscribed_at;
    }
}

#[actix_web::post("/subscriptions")]
pub async fn subscribe(pool: web::Data<PgPool>, form: web::Form<NewFormData>) -> HttpResponse {
    let query_result = insert_subscription(pool.get_ref(), form.into_inner()).await;
    match query_result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::put("/subscriptions/{id}")]
pub async fn put_subscription(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    form: web::Form<NewFormData>,
) -> HttpResponse {
    let query_result = update_subscription(pool.get_ref(), &id, form.into_inner()).await;
    match query_result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::get("/subscriptions")]
pub async fn list_subscriptions(pool: web::Data<PgPool>) -> HttpResponse {
    let query_result = fetch_all_subscriptions(pool.get_ref()).await;
    match query_result {
        Ok(posts) => HttpResponse::Ok().json(web::Json(posts)),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Inserts a subscription into the database.
pub async fn insert_subscription(
    pool: &PgPool,
    form: NewFormData,
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
}

/// Updates a subscription in the database.
pub async fn update_subscription(
    pool: &PgPool,
    id: &Uuid,
    form: NewFormData,
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
