use crate::common::spawn_test_app;
use actix_http::StatusCode;
use actix_web_demo::{
    api::client_context::ClientContext,
    model::user::{NewUser, User},
};

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_a_201_for_valid_form_data() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // Act
    let response = client
        .post(&format!("{}/api/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(StatusCode::CREATED, response.status());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/api/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[actix_rt::test]
async fn client_context_success() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/client_context", app.address))
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
        .post(format!("{}/api/users", &app.address))
        .json(&user)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, create_user_response.status());

    // Try to get token with wrong password
    let user: User = create_user_response.json().await.unwrap();
    {
        let response = client
            .post(format!("{}/api/login", &app.address))
            .basic_auth(&user.id.to_string(), Some("baz"))
            .send()
            .await
            .unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    // Get token
    let response = client
        .post(format!("{}/api/login", &app.address))
        .basic_auth(&user.id.to_string(), Some("bar"))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, response.status());

    // Fail with wrong token
    let token: String = response.text().await.unwrap();
    {
        let response = client
            .post(format!("{}/api/verify", &app.address))
            .json(&format!("{}kjqw12", token))
            .send()
            .await
            .unwrap();
        assert_eq!(StatusCode::FORBIDDEN, response.status());
    }

    // Verify token
    let response = client
        .post(format!("{}/api/verify", &app.address))
        .json(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, response.status());
}
