#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    missing_docs
)]

//! A demo web service implemented with actix web.

use crate::graphql::schema::create_schema;
use crate::grpc::account::AccountServiceImpl;
use crate::grpc::string::MyStringService;
use crate::infra::middleware::{DigestFilter, SignatureFilter};
use crate::infra::security::jwt::{jwt_interceptor, Role};
use crate::infra::{configuration, middleware, security};
use actix_cors::Cors;
use actix_web::web::{Json, Payload};
use actix_web::{dev::Server, web, App, HttpServer};
use actix_web::{Error, HttpResponse};
use actix_web_grants::proc_macro::has_roles;
use actix_web_httpauth::middleware::HttpAuthentication;
use grpc::account::generated::account_service_server::AccountServiceServer;
use grpc::string::generated::string_service_server::StringServiceServer;
use infra::error::AppError;
use paperclip::actix::{api_v2_operation, Apiv2Schema, OpenApiExt};
use paperclip::v2::models::{DefaultApiRaw, Info, SecurityScheme};
use rest::{
    client_context::client_context,
    health_check::health,
    token::{request_token, verify_token},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use tonic::service::interceptor;
use tracing_actix_web::TracingLogger;

pub mod graphql;
pub mod grpc;
pub mod infra;
pub mod model;
pub mod repository;
pub mod rest;

/// A common response type for services.
pub type AppResult<T> = Result<T, AppError>;

/// The database connection pool type used in the application.
pub type DbPool = PgPool;

/// The database transaction type.
pub type Tx = Transaction<'static, Postgres>;

/// Starts the gRPC server.
pub async fn run_grpc(addr: SocketAddr, db: DbPool) -> anyhow::Result<()> {
    tracing::info!("Starting gRPC server on address {}", addr);

    tonic::transport::Server::builder()
        .layer(interceptor(jwt_interceptor))
        .add_service(StringServiceServer::new(MyStringService::default()))
        .add_service(AccountServiceServer::new(AccountServiceImpl::new(db)))
        .serve(addr)
        .await
        .map_err(Into::into)
}

/// Starts a [`Server`].
pub fn run_actix(
    http_listener: TcpListener,
    https_listener: TcpListener,
    db_pool: DbPool,
) -> anyhow::Result<Server> {
    tracing::info!(
        "Starting application on address {} and {}",
        http_listener.local_addr()?,
        https_listener.local_addr()?
    );
    let pool = web::Data::new(db_pool.clone());
    let schema = Arc::new(create_schema(db_pool));
    let server = HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(security::jwt::validate_jwt);
        App::new()
            // Database pool
            .app_data(pool.clone())
            // Middleware to apply to all requests
            .wrap(middleware::RequestWrapper)
            .wrap(TracingLogger::default())
            // GraphQL
            .app_data(web::Data::from(schema.clone()))
            .service(graphql::graphql)
            .service(graphql::graphql_playground)
            .wrap(Cors::permissive())
            // Health check
            .service(health)
            .service(request_token)
            .service(verify_token)
            // Other
            .service(client_context)
            // Api
            .service(
                web::scope("/api")
                    .wrap(auth)
                    .configure(rest::account_api::account_config)
                    .configure(rest::user_api::user_config)
                    .configure(rest::transfer_api::transfer_config)
                    // Secure endpoints
                    .route("/user", web::get().to(user))
                    .route("/admin", web::get().to(admin)),
            )
            .service(
                web::scope("/signature")
                    .wrap(SignatureFilter)
                    .wrap(DigestFilter)
                    .route("", web::get().to(HttpResponse::Ok)),
            )
            .route("/echo", web::post().to(echo))
            .wrap_api_with_spec(openapi_spec())
            .service(
                paperclip::actix::web::resource("/pets")
                    .route(paperclip::actix::web::post().to(echo_pet)),
            )
            .with_json_spec_at("/spec/v2")
            .with_swagger_ui_at("/swagger-ui")
            .build()
    })
    .listen(http_listener)?
    .run();
    Ok(server)
}

/// Common configuration for entire API.
fn openapi_spec() -> DefaultApiRaw {
    let mut spec = DefaultApiRaw {
        info: Info {
            title: "actix-web-demo API".to_string(),
            description: Some("An API for managing users, accounts, and transactions".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    let security_scheme = SecurityScheme {
        type_: "apiKey".to_string(),
        in_: Some("header".to_string()),
        ..Default::default()
    };
    spec.security_definitions
        .insert("Authorization".to_string(), security_scheme);
    spec
}

#[has_roles("Role::User", type = "Role")]
async fn user() -> HttpResponse {
    HttpResponse::Ok().body("Hello user!".to_string())
}

#[has_roles("Role::Admin", type = "Role")]
async fn admin() -> HttpResponse {
    HttpResponse::Ok().body("Hello admin!".to_string())
}

async fn echo(payload: Payload) -> HttpResponse {
    HttpResponse::Ok().streaming(payload)
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
struct Pet {
    name: String,
    id: Option<i64>,
}

#[api_v2_operation]
async fn echo_pet(body: Json<Pet>) -> Result<Json<Pet>, Error> {
    Ok(body)
}
