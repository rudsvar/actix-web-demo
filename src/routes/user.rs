use crate::model::user::{HashedPassword, NewUser, User};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[actix_web::post("/users")]
pub async fn post_user(db: Data<PgPool>, new_user: Json<NewUser>) -> HttpResponse {
    let new_user = new_user.into_inner();
    // Convert to user model
    let user = User {
        id: Uuid::new_v4(),
        name: new_user.name,
        password: HashedPassword::new(new_user.password),
        created_at: Utc::now(),
    };
    // Store in db
    crate::db::user::store_user(db.get_ref(), &user)
        .await
        .unwrap();
    // Respond with newly created object
    HttpResponse::Created().json(user)
}

#[actix_web::get("/users/{id}")]
pub async fn get_user(db: Data<PgPool>, id: Path<Uuid>) -> HttpResponse {
    let user = crate::db::user::fetch_user_by_id(db.get_ref(), &id)
        .await
        .unwrap();
    HttpResponse::Ok().json(user)
}

#[actix_web::get("/users")]
pub async fn list_users(db: Data<PgPool>) -> HttpResponse {
    let users = crate::db::user::fetch_all_users(db.get_ref())
        .await
        .unwrap();
    HttpResponse::Ok().json(users)
}
