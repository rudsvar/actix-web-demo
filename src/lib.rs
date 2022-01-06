#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    missing_docs
)]

//! A demo web service implemented with actix web.

pub mod configuration;
pub mod db;
pub mod middleware;
pub mod model;
pub mod routes;
pub mod startup;
pub mod telemetry;
