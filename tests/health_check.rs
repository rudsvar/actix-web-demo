use actix_http::StatusCode;
use actix_web_demo::{
    configuration::{get_configuration, DatabaseSettings},
    routes::{ClientContext, NewFormData},
    startup::run,
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Spawn an application used for testing
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    configure_database(&configuration.database).await;
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!()
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
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
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // Act
    let response = client
        .post(&format!("{}/subscriptions", app.address))
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
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", app.address))
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
    let app = spawn_app().await;
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
async fn can_connect_to_db() {
    let configuration = get_configuration().expect("could not read configuration");
    let connection_string = configuration.database.connection_string();
    let _ = PgConnection::connect(&connection_string)
        .await
        .expect("failed to connect");
}

#[actix_rt::test]
async fn subscription_db_operations() {
    let mut configuration = get_configuration().expect("could not read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let pool = configure_database(&configuration.database).await;

    // Insert
    let form = NewFormData::new("foo@example.com", "foo");
    let inserted_form = actix_web_demo::routes::insert_subscription(&pool, form)
        .await
        .unwrap();
    assert_eq!("foo@example.com", inserted_form.email());
    assert_eq!("foo", inserted_form.name());

    // Fetch all
    let subscriptions = actix_web_demo::routes::fetch_all_subscriptions(&pool)
        .await
        .unwrap();
    assert_eq!(1, subscriptions.len());
    let form = &subscriptions[0];
    assert_eq!("foo@example.com", form.email());
    assert_eq!("foo", form.name());

    // Update
    let new_form = NewFormData::new("bar@example.com", "bar");
    let updated_form = actix_web_demo::routes::update_subscription(&pool, inserted_form.id(), new_form).await.unwrap();
    assert_ne!(inserted_form.email(), updated_form.email());
    assert_ne!(inserted_form.name(), updated_form.name());
    assert_eq!(inserted_form.id(), updated_form.id());
    assert_eq!(inserted_form.subscribed_at(), updated_form.subscribed_at());

    // Fetch all
    let subscriptions = actix_web_demo::routes::fetch_all_subscriptions(&pool)
        .await
        .unwrap();
    assert_eq!(1, subscriptions.len());
    let form = &subscriptions[0];
    assert_eq!("bar@example.com", form.email());
    assert_eq!("bar", form.name());
}
