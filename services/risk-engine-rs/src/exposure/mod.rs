use rust_decimal::Decimal;
use crate::{CorrelationEngine, RiskInputs};
use crate::error::RiskError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExposureMetrics {
    pub total_exposure: Decimal,
    pub total_positions: usize,
    pub long_exposure: Decimal,
    pub short_exposure: Decimal,
}

pub struct TotalExposure {
    pub value: Decimal,
}

pub struct ExposureEngine {
    max_positions: u32,
}

impl ExposureEngine {
    pub fn new(max_positions: u32) -> Self {
        Self { max_positions }
    }

    pub async fn calculate(&self, inputs: &RiskInputs, _correlation: &CorrelationEngine) -> Result<ExposureMetrics, RiskError> {
        let mut metrics = ExposureMetrics::default();
        
        metrics.total_positions = inputs.open_positions.len();
        if metrics.total_positions >= self.max_positions as usize {
            return Err(RiskError::invalid_input("max_positions", "Max positions reached"));
        }
        
        for (_, size, dir) in &inputs.open_positions {
            if *dir > 0 {
                metrics.long_exposure += size;
            } else {
                metrics.short_exposure += size;
            }
            metrics.total_exposure += size;
        }
        
        Ok(metrics)
    }
}
