//! Logging utilities.

use crate::{repository::audit_log_repository, DbPool};

use super::{
    audit_log::AuditLayer,
    configuration::{LogFormat, Settings},
};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Initialize logging facilitites.
pub async fn init_logging(config: &Settings, pool: DbPool) -> anyhow::Result<()> {
    let registry = tracing_subscriber::registry();

    // Add opentelemetry_jaeger layer
    let opentelemetry = if config.logging.opentelemetry {
        let opentelemetry_tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name(&config.application.name)
            .install_simple()?;
        let opentelemetry = tracing_opentelemetry::layer().with_tracer(opentelemetry_tracer);
        Some(opentelemetry)
    } else {
        None
    };
    let registry = registry.with(opentelemetry);

    // Add tokio-console tracing
    let console_layer = if config.logging.tokio_console {
        let console_layer = console_subscriber::spawn();
        Some(console_layer)
    } else {
        None
    };
    let registry = registry.with(console_layer);

    // Set up audit log layer and event listener
    let (tx, mut rc) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(msg) = rc.recv().await {
            let mut conn = pool.begin().await.unwrap();
            audit_log_repository::store_audit_event(&mut conn, &msg)
                .await
                .unwrap();
            conn.commit().await.unwrap();
        }
    });
    let audit_layer = AuditLayer::new(tx);
    let registry = registry.with(audit_layer);

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
