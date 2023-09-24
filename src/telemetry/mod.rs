use crate::config::CONFIG;
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;

pub fn init_telemetry() {
    init_logs();
    let _ = init_tracing();
}

fn init_logs() {
    env_logger::builder()
    .filter_level(CONFIG.log_level)
    .init();
}

fn init_tracing() -> Result<(), TraceError> {
    opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_endpoint("http://localhost:4317"))
    .install_batch(opentelemetry::runtime::Tokio)?;

    Ok(())
}