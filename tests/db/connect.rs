use actix_web_demo::configuration::get_configuration;
use sqlx::{Connection, PgConnection};

#[actix_rt::test]
async fn can_connect_to_db() {
    let configuration = get_configuration().expect("could not read configuration");
    let connection_string = configuration.database.connection_string();
    let _ = PgConnection::connect(&connection_string)
        .await
        .expect("failed to connect");
}
