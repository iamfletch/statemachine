//! Sets up tracing_subscriber to send OTLP
//!
//! # Setup
//! Add to start of application
//! ```no_run
//! use telemetry::Telemetry;
//! let _t = Telemetry::new();
//! ```
//!
//! .cargo/config.toml
//! ```toml
//! [env]
//! OTEL_SERVICE_NAME="service_name"
//! OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
//! RUST_LOG="info"
//! ```
//!

use opentelemetry::global;
use opentelemetry::runtime::Tokio;
use opentelemetry::trace::TraceError;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::fmt;

pub struct Telemetry;

pub mod log {
    pub use log::{trace, info, warn, error};
}

pub mod trace {
    pub use tracing::{trace, info, warn, error, trace_span, info_span, warn_span, error_span};
}


impl Telemetry {
    pub fn new() -> Self {
        let _ = init_tracer();
        Self
    }
}
impl Drop for Telemetry {
    fn drop(&mut self) {
        global::shutdown_tracer_provider();
        global::shutdown_logger_provider();
    }
}

fn init_tracer() -> Result<(), TraceError> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(Tokio)?;

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(fmt::Layer::default())
        .init();
    Ok(())
}