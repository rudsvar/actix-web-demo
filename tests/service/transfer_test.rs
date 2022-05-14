use actix_http::StatusCode;
use actix_web_demo::service::{
    account::account_model::Account, transfer::transfer_model::NewTransfer,
};
use reqwest::Client;

use crate::common::{spawn_test_app, TestApp};

#[actix_web::test]
async fn user_can_transfer_between_own_accounts() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let user_token = super::authenticate(&app, "user", "user").await;

    let new_transfer = NewTransfer {
        from_account: 1,
        to_account: 2,
        amount: 50,
    };

    let old_from = get_account(1, new_transfer.from_account, &user_token, &client, &app).await;
    let old_to = get_account(1, new_transfer.to_account, &user_token, &client, &app).await;

    let response = client
        .post(format!("{}/api/transfers", app.address()))
        .bearer_auth(&user_token)
        .json(&new_transfer)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, response.status());

    let new_from = get_account(1, new_transfer.from_account, &user_token, &client, &app).await;
    let new_to = get_account(1, new_transfer.to_account, &user_token, &client, &app).await;

    assert_eq!(
        old_from.balance() - new_transfer.amount as i64,
        new_from.balance()
    );
    assert_eq!(
        old_to.balance() + new_transfer.amount as i64,
        new_to.balance()
    );
}

#[actix_web::test]
async fn user_can_transfer_to_non_owned_account() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let user_token = super::authenticate(&app, "user", "user").await;
    let admin_token = super::authenticate(&app, "admin", "admin").await;

    let new_transfer = NewTransfer {
        from_account: 1,
        to_account: 3,
        amount: 50,
    };

    let old_from = get_account(1, new_transfer.from_account, &user_token, &client, &app).await;
    let old_to = get_account(2, new_transfer.to_account, &admin_token, &client, &app).await;

    let response = client
        .post(format!("{}/api/transfers", app.address()))
        .bearer_auth(&user_token)
        .json(&new_transfer)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, response.status());

    let new_from = get_account(1, new_transfer.from_account, &user_token, &client, &app).await;
    let new_to = get_account(2, new_transfer.to_account, &admin_token, &client, &app).await;

    assert_eq!(
        old_from.balance() - new_transfer.amount as i64,
        new_from.balance()
    );
    assert_eq!(
        old_to.balance() + new_transfer.amount as i64,
        new_to.balance()
    );
}

#[actix_web::test]
async fn user_cannot_transfer_from_non_owned_account() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let user_token = super::authenticate(&app, "user", "user").await;

    let new_transfer = NewTransfer {
        from_account: 3,
        to_account: 1,
        amount: 50,
    };

    let response = client
        .post(format!("{}/api/transfers", app.address()))
        .bearer_auth(&user_token)
        .json(&new_transfer)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::FORBIDDEN, response.status());
}

#[actix_web::test]
async fn admin_can_transfer_from_non_owned_account() {
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    let user_token = super::authenticate(&app, "admin", "admin").await;

    let new_transfer = NewTransfer {
        from_account: 1,
        to_account: 3,
        amount: 50,
    };

    let response = client
        .post(format!("{}/api/transfers", app.address()))
        .bearer_auth(&user_token)
        .json(&new_transfer)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::CREATED, response.status());
}

async fn get_account(
    user_id: i32,
    account_id: i32,
    token: &str,
    client: &Client,
    app: &TestApp,
) -> Account {
    client
        .get(format!(
            "{}/api/users/{}/accounts/{}",
            app.address(),
            user_id,
            account_id
        ))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
