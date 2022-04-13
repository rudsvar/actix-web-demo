use crate::common::spawn_test_app;
use actix_http::StatusCode;
use actix_web_demo::{
    api::client_context::ClientContext,
    service::user::user_model::{NewUser, User},
};

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", app.address()))
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn client_context_success() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/client_context", app.address()))
        .header("user_id", "5")
        .header("user_name", "frodo")
        .header("token", "qwerty12345")
        .send()
        .await
        .expect("request failed");
    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(
        ClientContext::new(5, "frodo", "qwerty12345"),
        response.json().await.unwrap()
    );
}

#[actix_rt::test]
async fn user_creation_to_token_verification() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    // Create new user
    let user = NewUser {
        name: "foo".to_string(),
        password: "bar".to_string(),
    };
    let create_user_response = client
        .post(format!("{}/api/users", &app.address()))
        .json(&user)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, create_user_response.status());

    // Try to get token with wrong password
    let user: User = create_user_response.json().await.unwrap();
    {
        let response = client
            .post(format!("{}/api/login", &app.address()))
            .basic_auth(&user.id.to_string(), Some("baz"))
            .send()
            .await
            .unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    // Get token
    let response = client
        .post(format!("{}/api/login", &app.address()))
        .basic_auth(&user.id.to_string(), Some("bar"))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, response.status());

    // Fail with wrong token
    let token: String = response.text().await.unwrap();
    {
        let response = client
            .post(format!("{}/api/verify", &app.address()))
            .json(&format!("{}kjqw12", token))
            .send()
            .await
            .unwrap();
        assert_eq!(StatusCode::FORBIDDEN, response.status());
    }

    // Verify token
    let response = client
        .post(format!("{}/api/verify", &app.address()))
        .json(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, response.status());
}
