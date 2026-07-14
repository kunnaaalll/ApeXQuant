use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    trace::{self, Sampler},
    Resource,
};
use anyhow::{Result, Context};
use metrics_exporter_prometheus::PrometheusBuilder;

pub fn init_telemetry(service_name: &str) -> Result<()> {
    // Initialize Prometheus Metrics
    PrometheusBuilder::new()
        .install()
        .context("Failed to install Prometheus metrics exporter")?;

    // Initialize OpenTelemetry Tracing
    let resource = Resource::new(vec![KeyValue::new("service.name", service_name.to_string())]);

    // Use stdout exporter if OTLP is not configured
    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_err() {
        tracing::warn!("OTEL_EXPORTER_OTLP_ENDPOINT not set. Tracing to console only.");
        return Ok(());
    }

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(resource),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .context("Failed to install OpenTelemetry tracer")?;

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    // We expect tracing_subscriber::fmt::init() was either called or we do it here.
    // If it's already initialized, this might fail, so we handle it gracefully or do it in main.
    // For now, we just return the tracer setup. The caller should compose the layers.
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(telemetry_layer)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap_or_else(|e| tracing::error!("Tracing init failed: {}", e));

    Ok(())
}

pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
}
