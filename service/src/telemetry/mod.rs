use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use crate::config::CONFIG;
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_telemetry() {
    let _ = init_tracing();
}

#[allow(dead_code)]
fn init_logs() {
    env_logger::builder()
    .filter_level(CONFIG.log_level)
    .init();
}

pub fn init_tracing() -> Result<(), TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(opentelemetry_otlp::new_exporter().tonic())
    .install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer);

    tracing_subscriber::registry()
        .with(telemetry_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    Ok(())
}