use actix_http::StatusCode;

use crate::common::spawn_test_app;

#[actix_rt::test]
async fn signed_request_works() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/signature", app.address()))
        .header("Authorization", r#"Signature keyId="abc123", algorithm="ecdsa-sha256", headers="(request-target)", signature="MEUCIQCDArOjrkKeWxtz012jVjfC7myJnGKPVngrnA6VS5356QIgStd+k9YK3jZ1vMg3BrfILdPXfBcrH5HMWrn5hkHRPi4="'"#)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::OK, response.status());
}

#[actix_rt::test]
async fn wrongly_signed_request_works() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/signature", app.address()))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::OK, response.status());
}
