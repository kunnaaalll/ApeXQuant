use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct SessionOutputs {
    pub priority_score: Decimal,
    pub edge_score: Decimal,
    pub confidence_score: Decimal,
    pub sample_adequacy: bool,
}

#[derive(Debug, Clone)]
pub struct SessionOptimizer {
    pub min_sample_size: u32,
}

impl SessionOptimizer {
    pub fn new(min_sample_size: u32) -> Self {
        Self { min_sample_size }
    }

    pub fn evaluate(&self, trades: u32, raw_edge: Decimal, confidence: Decimal) -> SessionOutputs {
        let sample_adequacy = trades >= self.min_sample_size;
        
        // Priority score is edge * confidence if sample is adequate
        let priority_score = if sample_adequacy {
            raw_edge * confidence
        } else {
            rust_decimal_macros::dec!(0.0)
        };

        SessionOutputs {
            priority_score,
            edge_score: raw_edge,
            confidence_score: confidence,
            sample_adequacy,
        }
    }
}
