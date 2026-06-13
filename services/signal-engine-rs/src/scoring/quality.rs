//! Signal quality metrics

/// Signal quality assessment
#[derive(Debug, Clone)]
pub struct SignalQuality {
    pub overall_score: f64,
    pub confidence: f64,
    pub reliability: f64,
}

impl SignalQuality {
    pub fn new(score: f64, confidence: f64) -> Self {
        Self {
            overall_score: score,
            confidence,
            reliability: score * confidence,
        }
    }
}
