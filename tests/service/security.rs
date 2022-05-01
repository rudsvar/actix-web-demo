use crate::common::spawn_test_app;
use actix_http::StatusCode;

#[actix_rt::test]
async fn user_can_access_user_endpoint() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/api/user", app.address()))
        .basic_auth("user", Some("user"))
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::OK, response.status());
}

#[actix_rt::test]
async fn user_cannot_access_admin_endpoint() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/api/admin", app.address()))
        .basic_auth("user", Some("user"))
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::FORBIDDEN, response.status());
}

#[actix_rt::test]
async fn admin_can_access_admin() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/api/admin", app.address()))
        .basic_auth("admin", Some("admin"))
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::OK, response.status());
}

#[actix_rt::test]
async fn admin_can_access_user() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/api/user", app.address()))
        .basic_auth("admin", Some("admin"))
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::OK, response.status());
}
