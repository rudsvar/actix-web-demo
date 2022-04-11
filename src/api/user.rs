//! Routes for user management.

use crate::{
    db::user::{fetch_all_users, fetch_user_by_id, store_user},
    error::ApplicationError,
    model::user::NewUser,
    DbPool,
};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};

#[actix_web::post("/users")]
pub async fn post_user(
    db: Data<DbPool>,
    new_user: Json<NewUser>,
) -> Result<HttpResponse, ApplicationError> {
    let new_user = new_user.into_inner();
    let user = store_user(db.get_ref(), &new_user).await?;
    Ok(HttpResponse::Created().json(user))
}

#[actix_web::get("/users/{id}")]
pub async fn get_user(db: Data<DbPool>, id: Path<i32>) -> Result<HttpResponse, ApplicationError> {
    let user = fetch_user_by_id(db.get_ref(), &id).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[actix_web::get("/users")]
pub async fn list_users(db: Data<DbPool>) -> Result<HttpResponse, ApplicationError> {
    let users = fetch_all_users(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(users))
}
