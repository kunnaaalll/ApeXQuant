use crate::latency::measurement::LatencyMeasurement;

pub struct DegradationDetector;

impl DegradationDetector {
    pub fn is_degraded(current: &LatencyMeasurement, baseline: &LatencyMeasurement) -> bool {
        // Simple logic: if current is more than 200% of baseline and > 50ms total
        let current_total = current.total_ms();
        let baseline_total = baseline.total_ms();

        if baseline_total == 0 {
            return false;
        }

        current_total > 50 && current_total > (baseline_total * 2)
    }
}
