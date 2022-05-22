use std::collections::HashMap;

use actix_http::StatusCode;
use actix_web_demo::security::signature::{self, SignatureHeader};

use crate::common::spawn_test_app;

#[actix_rt::test]
async fn signed_request_works() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    let headers_to_sign = vec!["(request-target)"];

    let private_key = signature::load_private_key("./keys/private.pem");
    let mut headers = HashMap::new();
    headers.insert("(request-header)", vec!["get /signature"]);
    let signature_string = signature::signature_string(&headers_to_sign, &headers);
    let signature = signature::sign(signature_string.as_bytes(), private_key);
    let base64_signature = base64::encode(&signature);
    let signature_header = SignatureHeader::new(
        "public".to_string(),
        "ecdsa-sha256".to_string(),
        headers_to_sign
            .iter()
            .cloned()
            .map(|s| s.to_string())
            .collect(),
        base64_signature,
    );

    let response = client
        .get(format!("{}/signature", app.address()))
        .header("Authorization", signature_header.to_string())
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::OK, response.status());
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
