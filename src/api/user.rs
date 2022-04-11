//! Routes for user management.

use crate::{model::user::NewUser, DbPool};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};

#[actix_web::post("/users")]
pub async fn post_user(db: Data<DbPool>, new_user: Json<NewUser>) -> HttpResponse {
    let new_user = new_user.into_inner();
    // Store in db
    let user = crate::db::user::store_user(db.get_ref(), &new_user)
        .await
        .unwrap();
    // Respond with newly created object
    HttpResponse::Created().json(user)
}

#[actix_web::get("/users/{id}")]
pub async fn get_user(db: Data<DbPool>, id: Path<i32>) -> HttpResponse {
    let user = crate::db::user::fetch_user_by_id(db.get_ref(), &id)
        .await
        .unwrap();
    HttpResponse::Ok().json(user)
}

#[actix_web::get("/users")]
pub async fn list_users(db: Data<DbPool>) -> HttpResponse {
    let users = crate::db::user::fetch_all_users(db.get_ref())
        .await
        .unwrap();
    HttpResponse::Ok().json(users)
}
