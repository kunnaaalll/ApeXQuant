use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenLeverageAssessment {
    pub has_hidden_leverage: bool,
    pub synthetic_duplicates: Vec<SyntheticDuplicate>,
    pub theme_concentration: Vec<ThemeConcentration>,
    pub total_hidden_leverage_ratio: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticDuplicate {
    pub symbols: Vec<String>,
    pub correlation_score: Decimal,
    pub combined_exposure_pct: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConcentration {
    pub theme: String,
    pub symbols: Vec<String>,
    pub total_exposure_pct: Decimal,
}

impl Default for HiddenLeverageAssessment {
    fn default() -> Self {
        Self::new()
    }
}

impl HiddenLeverageAssessment {
    pub fn new() -> Self {
        Self {
            has_hidden_leverage: false,
            synthetic_duplicates: Vec::new(),
            theme_concentration: Vec::new(),
            total_hidden_leverage_ratio: Decimal::ZERO,
        }
    }

    pub fn assess(&mut self) {
        // Evaluate flags
        self.has_hidden_leverage = !self.synthetic_duplicates.is_empty() || !self.theme_concentration.is_empty();
        
        let mut total_ratio = Decimal::ZERO;
        for dup in &self.synthetic_duplicates {
            total_ratio += dup.combined_exposure_pct * dup.correlation_score;
        }
        self.total_hidden_leverage_ratio = total_ratio;
    }
}
