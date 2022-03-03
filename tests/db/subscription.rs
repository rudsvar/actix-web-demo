use crate::common::test_db;
use actix_web_demo::{configuration::get_configuration, model::subscription::NewSubscription};

#[actix_rt::test]
async fn subscription_db_operations() {
    let configuration = get_configuration().expect("could not read configuration");
    let pool = test_db(configuration.database).await;

    // Insert
    let form = NewSubscription::new("foo@example.com", "foo");
    let inserted_form = actix_web_demo::db::subscription::insert_subscription(&pool, &form)
        .await
        .unwrap();
    assert_eq!("foo@example.com", inserted_form.email);
    assert_eq!("foo", inserted_form.name);

    // Fetch all
    let subscriptions = actix_web_demo::db::subscription::fetch_all_subscriptions(&pool)
        .await
        .unwrap();
    assert_eq!(1, subscriptions.len());
    let form = &subscriptions[0];
    assert_eq!("foo@example.com", form.email);
    assert_eq!("foo", form.name);

    // Update
    let new_form = NewSubscription::new("bar@example.com", "bar");
    let updated_form =
        actix_web_demo::db::subscription::update_subscription(&pool, &inserted_form.id, &new_form)
            .await
            .unwrap();
    assert_ne!(inserted_form.email, updated_form.email);
    assert_ne!(inserted_form.name, updated_form.name);
    assert_eq!(inserted_form.id, updated_form.id);
    assert_eq!(inserted_form.subscribed_at, updated_form.subscribed_at);

    // Fetch all
    let subscriptions = actix_web_demo::db::subscription::fetch_all_subscriptions(&pool)
        .await
        .unwrap();
    assert_eq!(1, subscriptions.len());
    let form = &subscriptions[0];
    assert_eq!("bar@example.com", form.email);
    assert_eq!("bar", form.name);
}
