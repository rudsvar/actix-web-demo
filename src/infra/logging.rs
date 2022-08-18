//! Logging utilities.
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Initialize logging facilitites.
pub fn init_logging() {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("actix-web-demo")
        .install_simple()
        .unwrap();
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let log_format = std::env::var("LOG_FORMAT").ok();
    let log_format = log_format.as_deref();
    let console_layer = console_subscriber::spawn();
    let registry = tracing_subscriber::registry()
        .with(opentelemetry.with_filter(EnvFilter::from_default_env()))
        .with(console_layer);

    match log_format {
        Some("bunyan") => {
            let json_storage_layer = JsonStorageLayer;
            let bunyan_layer =
                BunyanFormattingLayer::new("actix-web-demo".to_string(), std::io::stdout)
                    .with_filter(EnvFilter::from_default_env());
            registry.with(json_storage_layer).with(bunyan_layer).init();
        }
        Some("json") => {
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_filter(EnvFilter::from_default_env());
            registry.with(json_layer).init();
        }
        _ => {
            let fmt_layer =
                tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env());
            registry.with(fmt_layer).init();
        }
    }
}
