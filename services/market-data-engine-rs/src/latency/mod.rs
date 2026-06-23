use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LatencyGrade {
    Excellent,
    Normal,
    Slow,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatencyMetrics {
    pub source_latency_ms: Decimal,
    pub network_latency_ms: Decimal,
    pub processing_latency_ms: Decimal,
}

impl LatencyMetrics {
    pub fn new(source: Decimal, network: Decimal, processing: Decimal) -> Self {
        Self {
            source_latency_ms: source,
            network_latency_ms: network,
            processing_latency_ms: processing,
        }
    }

    pub fn total_latency_ms(&self) -> Decimal {
        self.source_latency_ms + self.network_latency_ms + self.processing_latency_ms
    }

    pub fn grade(&self) -> LatencyGrade {
        let total = self.total_latency_ms();
        if total <= Decimal::from(5) {
            LatencyGrade::Excellent
        } else if total <= Decimal::from(20) {
            LatencyGrade::Normal
        } else if total <= Decimal::from(100) {
            LatencyGrade::Slow
        } else {
            LatencyGrade::Critical
        }
    }
}
