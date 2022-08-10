use crate::common::spawn_test_app;
use actix_http::{header::HttpDate, StatusCode};
use actix_web_demo::infra::security::signature::{self, Algorithm, Headers};
use openssl::hash::MessageDigest;
use std::time::SystemTime;

#[actix_web::test]
async fn request_with_digest_works() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let date = HttpDate::from(SystemTime::now()).to_string();

    let body = "hello!";
    let digested_body = openssl::hash::hash(MessageDigest::sha256(), body.as_bytes()).unwrap();
    let digested_body = openssl::base64::encode_block(&digested_body);
    let digest = format!("sha256={}", digested_body);

    let mut headers = Headers::new();
    headers.add("(request-target)", "get /signature");
    headers.add("date", &date);
    headers.add("digest", &digest);

    let private_key =
        signature::load_private_key("./resources/test-signing-key.pem", &Algorithm::EcdsaSha256)
            .unwrap();
    let signature_header =
        signature::signature_header("test", Algorithm::EcdsaSha256, &headers, private_key).unwrap();

    let response = client
        .get(format!("{}/signature", app.address()))
        .header("Authorization", signature_header.to_string())
        .header("Date", date)
        .header("Digest", digest)
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::OK, response.status());
}

#[actix_web::test]
async fn request_with_wrong_digest_fails() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let date = HttpDate::from(SystemTime::now()).to_string();

    let body = "hello!";
    let not_body = "not hello!";
    let not_digest_body =
        openssl::hash::hash(MessageDigest::sha256(), not_body.as_bytes()).unwrap();
    let not_digest_body = openssl::base64::encode_block(&not_digest_body);

    let mut headers = Headers::new();
    headers.add("(request-target)", "get /signature");
    headers.add("date", &date);
    headers.add("digest", &not_digest_body);

    let private_key =
        signature::load_private_key("./resources/test-signing-key.pem", &Algorithm::EcdsaSha256)
            .unwrap();
    let signature_header =
        signature::signature_header("test", Algorithm::EcdsaSha256, &headers, private_key).unwrap();

    let response = client
        .get(format!("{}/signature", app.address()))
        .header("Authorization", signature_header.to_string())
        .header("Date", date)
        .header("Digest", format!("sha256={}", not_digest_body))
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

#[actix_web::test]
async fn request_with_body_but_no_digest_fails() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let date = HttpDate::from(SystemTime::now()).to_string();

    let mut headers = Headers::new();
    headers.add("(request-target)", "get /signature");
    headers.add("date", &date);

    let private_key =
        signature::load_private_key("./resources/test-signing-key.pem", &Algorithm::EcdsaSha256)
            .unwrap();
    let signature_header =
        signature::signature_header("test", Algorithm::EcdsaSha256, &headers, private_key).unwrap();

    let body = "hello!";
    let response = client
        .get(format!("{}/signature", app.address()))
        .header("Authorization", signature_header.to_string())
        .header("Date", date)
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

#[actix_web::test]
async fn not_signing_digest_fails() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let date = HttpDate::from(SystemTime::now()).to_string();

    let body = "hello!";
    let digest_body = openssl::hash::hash(MessageDigest::sha256(), body.as_bytes()).unwrap();
    let digest_body = openssl::base64::encode_block(&digest_body);

    let mut headers = Headers::new();
    headers.add("(request-target)", "get /signature");
    headers.add("date", &date);

    let private_key =
        signature::load_private_key("./resources/test-signing-key.pem", &Algorithm::EcdsaSha256)
            .unwrap();
    let signature_header =
        signature::signature_header("test", Algorithm::EcdsaSha256, &headers, private_key).unwrap();

    let response = client
        .get(format!("{}/signature", app.address()))
        .header("Authorization", signature_header.to_string())
        .header("Date", date)
        .header("Digest", format!("sha256={}", digest_body))
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}
