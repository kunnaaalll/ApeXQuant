pub struct DatasetEngine;

impl DatasetEngine {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn prepare_dataset(&self, timeframe: &str, regime: &str) -> String {
        // Deterministic string representation of a prepared dataset
        format!("dataset_v1_{}_{}", timeframe, regime)
    }
}
