use actix_http::StatusCode;
use actix_web_demo::service::user::user_model::{NewUser, User};

use crate::common::spawn_test_app;

#[actix_web::test]
async fn creating_user_adds_it_to_db() {
    let app = spawn_test_app().await;
    let db = app.db();
    let client = reqwest::Client::new();
    let admin_token = super::authenticate(&app, "admin", "admin").await;

    let new_user = NewUser {
        name: "foo".to_string(),
        password: "bar".to_string(),
    };

    let response = client
        .post(format!("{}/api/users", app.address()))
        .bearer_auth(admin_token)
        .json(&new_user)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, response.status());
    let user_from_response: User = response.json().await.unwrap();

    let user_from_db = sqlx::query!("SELECT * FROM users WHERE id = $1", user_from_response.id)
        .fetch_one(db)
        .await
        .unwrap();

    assert_eq!(new_user.name, user_from_db.name);
    assert_ne!(new_user.password, user_from_db.password);

    // Log in
    let response = client
        .post(format!("{}/token", app.address()))
        .basic_auth("foo", Some("bar"))
        .json(&new_user)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, response.status());
}
