//! Logging utilities.

use super::configuration::{LogFormat, Settings};
use std::str::FromStr;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Initialize logging facilitites.
pub fn init_logging(config: &Settings) -> anyhow::Result<()> {
    let registry = tracing_subscriber::registry();

    // Add opentelemetry_jaeger layer
    let opentelemetry_filter = EnvFilter::from_str(&config.logging.opentelemetry)?;
    let opentelemetry_tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(&config.application.name)
        .install_simple()?;
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(opentelemetry_tracer);
    let registry = registry.with(opentelemetry.with_filter(opentelemetry_filter));

    // Add tokio-console tracing
    let console_layer_filter = EnvFilter::from_str(&config.logging.tokio_console)?;
    let console_layer = console_subscriber::spawn().with_filter(console_layer_filter);
    let registry = registry.with(console_layer);

    match config.logging.format {
        LogFormat::Bunyan => {
            let json_storage_layer = JsonStorageLayer;
            let bunyan_layer =
                BunyanFormattingLayer::new(config.application.name.clone(), std::io::stdout)
                    .with_filter(EnvFilter::from_default_env());
            registry
                .with(json_storage_layer)
                .with(bunyan_layer)
                .try_init()?;
        }
        LogFormat::Json => {
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_filter(EnvFilter::from_default_env());
            registry.with(json_layer).try_init()?;
        }
        LogFormat::Text => {
            let fmt_layer =
                tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env());
            registry.with(fmt_layer).try_init()?;
        }
    }

    Ok(())
}
