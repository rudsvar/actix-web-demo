//! Telemetry setup.

use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

/// Returns a subscriber that defines how logs should be made.
/// The filter says which level logs should be let through, and the two latter give information
/// about additional metadata and its format.
pub fn get_subscriber(
    name: String,
    env_filter: String,
    sink: impl for<'a> MakeWriter<'a> + Send + Sync + 'static,
) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Initializes the logger [`LogTracer`], and sets the global subscriber.
/// This should only be called once.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("failed to set logger");
    set_global_default(subscriber).expect("failed to set subscriber");
}
