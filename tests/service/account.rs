use crate::common::spawn_test_app;
use actix_http::StatusCode;
use actix_web_demo::service::account::{Account, Deposit, NewAccount, Withdrawal};

#[actix_rt::test]
async fn post_account_gives_201() {
    // Arrange
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let new_account = NewAccount::new("my_account".to_string(), 1);

    // Act
    let response = client
        .post(format!("{}/api/accounts", app.address()))
        .basic_auth("user", Some("user"))
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
    let response = client
        .get(format!("{}/api/accounts/4", app.address()))
        .basic_auth("user", Some("user"))
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
        .basic_auth("user", Some("user"))
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

    // Check old account status
    let old_account: Account = client
        .get(format!("{}/api/accounts/4", app.address()))
        .basic_auth("user", Some("user"))
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
        .basic_auth("user", Some("user"))
        .json(&deposit)
        .send()
        .await
        .unwrap();

    assert_eq!(StatusCode::CREATED, response.status());

    let new_account: Account = client
        .get(format!("{}/api/accounts/4", app.address()))
        .basic_auth("user", Some("user"))
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

    // Check old account status
    let old_account: Account = client
        .get(format!("{}/api/accounts/4", app.address()))
        .basic_auth("user", Some("user"))
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
        .basic_auth("user", Some("user"))
        .json(&withdrawal)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, response.status());

    let new_account: Account = client
        .get(format!("{}/api/accounts/4", app.address()))
        .basic_auth("user", Some("user"))
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
        .basic_auth("user", Some("user"))
        .json(&withdrawal)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::BAD_REQUEST, response.status());
}
