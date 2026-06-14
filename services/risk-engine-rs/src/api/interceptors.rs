use tonic::{Request, Status};
use tracing::Span;

/// Intercepts requests to extract correlation IDs and track basic metrics
pub fn metrics_interceptor(mut request: Request<()>) -> Result<Request<()>, Status> {
    let metadata = request.metadata();

    // Extract trace ID or create one
    let trace_id = if let Some(trace_val) = metadata.get("x-request-id") {
        trace_val.to_str().unwrap_or_default().to_string()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    // Attach to current tracing span if any
    let span = Span::current();
    span.record("trace_id", &trace_id);

    // Re-insert standard correlation ID if needed by downstream
    request
        .metadata_mut()
        .insert("x-request-id", trace_id.parse().unwrap());

    Ok(request)
}
