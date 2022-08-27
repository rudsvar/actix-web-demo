use super::authenticate;
use crate::common::spawn_test_app;
use actix_http::StatusCode;

#[actix_web::test]
async fn user_can_access_user_endpoint() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let token = authenticate(&app, "user", "user").await;

    // Act
    let response = client
        .get(format!("{}/api/user", app.address()))
        .bearer_auth(token)
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::OK, response.status());
}

#[actix_web::test]
async fn user_cannot_access_admin_endpoint() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let token = authenticate(&app, "user", "user").await;

    // Act
    let response = client
        .get(format!("{}/api/admin", app.address()))
        .bearer_auth(token)
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::FORBIDDEN, response.status());
}

#[actix_web::test]
async fn admin_can_access_admin() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let token = authenticate(&app, "admin", "admin").await;

    // Act
    let response = client
        .get(format!("{}/api/admin", app.address()))
        .bearer_auth(token)
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::OK, response.status());
}

#[actix_web::test]
async fn admin_can_access_user() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let token = authenticate(&app, "admin", "admin").await;

    // Act
    let response = client
        .get(format!("{}/api/user", app.address()))
        .bearer_auth(token)
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::OK, response.status());
}
