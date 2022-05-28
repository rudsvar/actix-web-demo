use crate::common::spawn_test_app;
use actix_http::{header::HttpDate, StatusCode};
use actix_web_demo::security::signature::{self, Algorithm, Headers, SignatureHeader};
use std::time::SystemTime;

#[actix_rt::test]
async fn signed_request_works() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let date = HttpDate::from(SystemTime::now()).to_string();

    let mut headers = Headers::new();
    headers.add("(request-target)", "get /signature");
    headers.add("date", &date);

    let private_key =
        signature::load_private_key("./tests/test-signing-key.pem", &Algorithm::EcdsaSha256)
            .unwrap();
    let signature_header =
        signature::signature_header("test", Algorithm::EcdsaSha256, &headers, private_key).unwrap();

    let response = client
        .get(format!("{}/signature", app.address()))
        .header("Authorization", signature_header.to_string())
        .header("Date", date)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::OK, response.status());
}

#[actix_rt::test]
async fn edited_signed_request_fails() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let date = HttpDate::from(SystemTime::now()).to_string();

    let mut headers = Headers::new();
    headers.add("(request-target)", "get /not-signature");
    headers.add("date", &date);

    let private_key =
        signature::load_private_key("./tests/test-signing-key.pem", &Algorithm::EcdsaSha256)
            .unwrap();
    let signature_header =
        signature::signature_header("test", Algorithm::EcdsaSha256, &headers, private_key).unwrap();

    let response = client
        .get(format!("{}/signature", app.address()))
        .header("Authorization", signature_header.to_string())
        .header("Date", date)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

#[actix_rt::test]
async fn signed_with_wrong_key_fails() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let date = HttpDate::from(SystemTime::now()).to_string();

    let mut headers = Headers::new();
    headers.add("(request-target)", "get /signature");
    headers.add("date", &date);

    let private_key = signature::load_private_key(
        "./tests/wrong-test-signing-key.pem",
        &Algorithm::EcdsaSha256,
    )
    .unwrap();
    let signature_header =
        signature::signature_header("test", Algorithm::EcdsaSha256, &headers, private_key).unwrap();

    let response = client
        .get(format!("{}/signature", app.address()))
        .header("Authorization", signature_header.to_string())
        .header("Date", date)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

#[actix_rt::test]
async fn invalid_signature_string_fails() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let signature_header = SignatureHeader::new(
        "test".to_string(),
        Algorithm::EcdsaSha256,
        vec!["(request-target)".to_string(), "date".to_string()],
        "not-a-signature".to_string(),
    );

    let response = client
        .get(format!("{}/signature", app.address()))
        .header("Authorization", signature_header.to_string())
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::BAD_REQUEST, response.status());
}

#[actix_rt::test]
async fn unsigned_request_fails() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/signature", app.address()))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}
