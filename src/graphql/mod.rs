//! A GraphQL API.

pub mod schema;

use crate::graphql::schema::Schema;
use actix_web::{web, HttpResponse, Responder};
use actix_web_lab::respond::Html;
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};

/// GraphiQL playground UI
#[actix_web::get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    Html(graphiql_source("/graphql", None))
}

/// GraphQL endpoint
#[actix_web::route("/graphql", method = "GET", method = "POST")]
async fn graphql(st: web::Data<Schema>, data: web::Json<GraphQLRequest>) -> impl Responder {
    let user = data.execute(&st, &()).await;
    HttpResponse::Ok().json(user)
}
