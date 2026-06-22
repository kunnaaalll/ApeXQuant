use metrics::{counter, histogram, gauge};
use std::time::Duration;

pub fn record_request(method: &str) {
    counter!("grpc_requests_total", "method" => method.to_string()).increment(1);
    gauge!("grpc_active_requests", "method" => method.to_string()).increment(1.0);
}

pub fn record_response(method: &str, status: &str, duration: Duration) {
    counter!("grpc_responses_total", "method" => method.to_string(), "status" => status.to_string()).increment(1);
    histogram!("grpc_request_duration_seconds", "method" => method.to_string()).record(duration.as_secs_f64());
    gauge!("grpc_active_requests", "method" => method.to_string()).decrement(1.0);
}
