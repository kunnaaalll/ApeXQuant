use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkResult {
    pub average_latency_ms: Decimal,
    pub p99_latency_ms: Decimal,
    pub replay_time_ms: Decimal,
    pub snapshot_time_ms: Decimal,
    pub serialization_time_ms: Decimal,
    pub validation_time_ms: Decimal,
}

pub struct BenchmarkEngine;

impl BenchmarkEngine {
    pub fn check_thresholds(result: &BenchmarkResult) -> bool {
        let max_avg = dec!(2);
        let max_p99 = dec!(10);

        result.average_latency_ms <= max_avg && result.p99_latency_ms <= max_p99
    }
}
