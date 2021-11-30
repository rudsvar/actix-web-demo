#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:8080/health_check")
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() {
    let server = actix_web_demo::run().expect("failed to start server");
    let _ = tokio::spawn(server);
}
