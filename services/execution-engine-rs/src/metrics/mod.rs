use metrics::{counter, histogram};

pub mod shadow;

pub fn record_order_placement() {
    counter!("execution.order.placed").increment(1);
}

pub fn record_execution_latency(duration_ms: f64) {
    histogram!("execution.latency.ms").record(duration_ms);
}
