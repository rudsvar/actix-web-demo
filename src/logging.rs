//! Logging utilities.
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Initialize logging facilitites.
pub fn init_logging() {
    let console_layer = console_subscriber::spawn();
    let registry = tracing_subscriber::registry().with(console_layer);
    if cfg!(debug_assertions) {
        let fmt_layer = tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env());
        registry.with(fmt_layer).init();
    } else {
        let json_storage_layer = JsonStorageLayer;
        let bunyan_layer =
            BunyanFormattingLayer::new("actix-web-demo".to_string(), std::io::stdout)
                .with_filter(EnvFilter::from_default_env());
        registry.with(json_storage_layer).with(bunyan_layer).init();
    }
}
