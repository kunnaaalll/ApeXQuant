use prometheus::{Encoder, Registry, TextEncoder};

pub struct MetricsRegistry {
    pub registry: Registry,
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRegistry {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
        }
    }

    pub fn gather(&self) -> String {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap_or(());
        String::from_utf8(buffer).unwrap_or_default()
    }
}
