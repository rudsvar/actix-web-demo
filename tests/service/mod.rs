use crate::common::TestApp;

mod account_test;
mod auth_test;
mod security_test;

pub async fn authenticate(app: &TestApp, username: &str, password: &str) -> String {
    let client = reqwest::Client::new();
    let token = client
        .post(format!("{}/token", app.address()))
        .basic_auth(username, Some(password))
        .send()
        .await
        .expect("failed to login")
        .text()
        .await
        .unwrap();
    token
}
