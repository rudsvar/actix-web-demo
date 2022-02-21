//! Routes for subscribing.

use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::subscription::{fetch_all_subscriptions, insert_subscription, update_subscription},
    model::subscription::NewSubscription,
};

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name= %form.name
        )
    )]
#[actix_web::post("/subscriptions")]
pub async fn post_subscription(
    pool: web::Data<PgPool>,
    form: web::Form<NewSubscription>,
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
    form: web::Form<NewSubscription>,
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
