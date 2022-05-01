use crate::common::spawn_test_app;
use actix_http::StatusCode;
use actix_web_demo::service::client_context::ClientContext;

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

    let username = "user";
    let password = Some("user");

    // Try to get token with wrong password
    {
        let response = client
            .post(format!("{}/login", &app.address()))
            .basic_auth(username, Some("wrongpassword"))
            .send()
            .await
            .unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    // Get token
    let response = client
        .post(format!("{}/login", &app.address()))
        .basic_auth(username, password)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, response.status());

    // Fail with invalid token
    {
        let response = client
            .post(format!("{}/verify", &app.address()))
            .json("kjqw12")
            .send()
            .await
            .unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    // Fail with expired token
    {
        let response = client
            .post(format!("{}/verify", &app.address()))
            .json("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6MSwiZXhwIjoxNjUxNDE4MDE3LCJyb2xlcyI6WyJVc2VyIl19.6dEgUhl2-rRNBiQRjiZ_4YDOFv2uHbAkolPlAk0v_TA")
            .send()
            .await
            .unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    let token: String = response.text().await.unwrap();
    // Verify token
    let response = client
        .post(format!("{}/verify", &app.address()))
        .json(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, response.status());
}
