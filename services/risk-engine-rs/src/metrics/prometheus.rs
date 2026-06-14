use metrics::{counter, gauge, histogram};
use std::time::Duration;

/// Record the latency of a risk assessment
pub fn record_assessment_latency(latency: Duration) {
    histogram!("risk_engine_assessment_latency_ms").record(latency.as_secs_f64() * 1000.0);
}

/// Record when a circuit breaker trips
pub fn record_circuit_breaker_trip(breaker_name: &str) {
    counter!("risk_engine_circuit_breaker_trips", "breaker" => breaker_name.to_string()).increment(1);
}

/// Record when a risk limit is exceeded
pub fn record_limit_exceeded(limit_type: &str) {
    counter!("risk_engine_limits_exceeded", "limit" => limit_type.to_string()).increment(1);
}

/// Update the active exposure gauge for a specific symbol
pub fn update_active_exposure(symbol: &str, amount: f64) {
    gauge!("risk_engine_active_exposures", "symbol" => symbol.to_string()).set(amount);
}
