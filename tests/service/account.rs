use crate::common::spawn_test_app;
use actix_http::StatusCode;
use actix_web_demo::service::{
    account::account_model::{Account, NewAccount},
    deposit::deposit_model::Deposit,
    withdrawal::withdrawal_model::Withdrawal,
};

#[actix_rt::test]
async fn post_account_gives_201() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let new_account = NewAccount::new("my_account".to_string(), 1);

    // Act
    let user_token = super::authenticate(&app, "user", "user").await;
    let response = client
        .post(format!("{}/api/accounts", app.address()))
        .bearer_auth(user_token)
        .json(&new_account)
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert_eq!(StatusCode::CREATED, response.status());

    let created_account: Account = response.json().await.unwrap();
    assert_ne!(0, created_account.id());
    assert_eq!("my_account".to_string(), created_account.name());
    assert_eq!(0, created_account.balance());
    assert_eq!(1, created_account.owner_id());
}

#[actix_rt::test]
async fn get_account_gives_200() {
    let app = spawn_test_app().await;
    // Populate db with test data
    sqlx::query_file!("tests/sql/accounts.sql")
        .execute(app.db())
        .await
        .unwrap();
    let client = reqwest::Client::new();

    // Read response
    let user_token = super::authenticate(&app, "user", "user").await;
    let response = client
        .get(format!("{}/api/users/1/accounts/4", app.address()))
        .bearer_auth(user_token)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, response.status());

    let account: Account = response.json().await.unwrap();
    assert_eq!(Account::new(4, "test".to_string(), 200, 1), account);
}

#[actix_rt::test]
async fn get_missing_account_gives_404() {
    let app = spawn_test_app().await;
    // Populate db with test data
    sqlx::query_file!("tests/sql/accounts.sql")
        .execute(app.db())
        .await
        .unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/accounts/0", app.address()))
        .bearer_auth("invalid_token")
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::NOT_FOUND, response.status());
}

#[actix_rt::test]
async fn deposit_increases_balance() {
    let app = spawn_test_app().await;
    sqlx::query_file!("tests/sql/accounts.sql")
        .execute(app.db())
        .await
        .unwrap();
    let client = reqwest::Client::new();
    let user_token = super::authenticate(&app, "user", "user").await;

    // Check old account status
    let old_account: Account = client
        .get(format!("{}/api/users/1/accounts/4", app.address()))
        .bearer_auth(&user_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Make a deposit
    let deposit_amount = 50;
    let deposit = Deposit::new(deposit_amount);
    let response = client
        .post(format!("{}/api/accounts/4/deposits", app.address()))
        .bearer_auth(&user_token)
        .json(&deposit)
        .send()
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, response.status());

    let new_account: Account = client
        .get(format!("{}/api/users/1/accounts/4", app.address()))
        .bearer_auth(&user_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(
        old_account.balance() + deposit_amount as i64,
        new_account.balance()
    );
}

#[actix_rt::test]
async fn withdraw_decreases_balance() {
    let app = spawn_test_app().await;
    sqlx::query_file!("tests/sql/accounts.sql")
        .execute(app.db())
        .await
        .unwrap();
    let client = reqwest::Client::new();
    let user_token = super::authenticate(&app, "user", "user").await;

    // Check old account status
    let old_account: Account = client
        .get(format!("{}/api/users/1/accounts/4", app.address()))
        .bearer_auth(&user_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Make a withdrawal
    let withdrawal_amount = 50;
    let withdrawal = Withdrawal::new(withdrawal_amount);
    let response = client
        .post(format!("{}/api/accounts/4/withdrawals", app.address()))
        .bearer_auth(&user_token)
        .json(&withdrawal)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, response.status());

    let new_account: Account = client
        .get(format!("{}/api/users/1/accounts/4", app.address()))
        .bearer_auth(&user_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(
        old_account.balance() - withdrawal_amount as i64,
        new_account.balance()
    );
}

#[actix_rt::test]
async fn withdrawing_too_much_fails() {
    let app = spawn_test_app().await;
    sqlx::query_file!("tests/sql/accounts.sql")
        .execute(app.db())
        .await
        .unwrap();
    let client = reqwest::Client::new();

    // Make a withdrawal
    let withdrawal_amount = 500;
    let withdrawal = Withdrawal::new(withdrawal_amount);
    let response = client
        .post(format!("{}/api/accounts/4/withdrawals", app.address()))
        .bearer_auth("invalid_token")
        .json(&withdrawal)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::BAD_REQUEST, response.status());
}

#[actix_web::test]
async fn user_cannot_access_other_account() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();

    // Get user's account as user
    let user_token = super::authenticate(&app, "user", "user").await;
    let response = client
        .get(format!("{}/api/users/1/accounts/1", app.address()))
        .bearer_auth(&user_token)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, response.status());

    // Get admin's account as user
    let response = client
        .get(format!("{}/api/users/2/accounts/3", app.address()))
        .bearer_auth(&user_token)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::FORBIDDEN, response.status());

    // Get user's account as admin
    let admin_token = super::authenticate(&app, "admin", "admin").await;
    let response = client
        .get(format!("{}/api/users/1/accounts/1", app.address()))
        .bearer_auth(&admin_token)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, response.status());

    // Get admin's account as admin
    let admin_token = super::authenticate(&app, "admin", "admin").await;
    let response = client
        .get(format!("{}/api/users/2/accounts/3", app.address()))
        .bearer_auth(admin_token)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, response.status());
}
