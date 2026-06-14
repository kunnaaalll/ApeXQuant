use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::confidence::penalties::ConfidencePenalty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    VeryLow,
    Low,
    Moderate,
    High,
    VeryHigh,
}

#[derive(Debug, Clone)]
pub struct ConfidenceScore {
    pub raw_score: Decimal, // 0-100
    pub final_score: Decimal, // 0-100
    pub level: ConfidenceLevel,
    pub penalties: Vec<ConfidencePenalty>,
    
    pub edge_score: Decimal,
    pub expectancy_score: Decimal,
    pub sample_adequacy: Decimal,
    pub regime_score: Decimal,
    pub session_score: Decimal,
    pub symbol_score: Decimal,
    pub timeframe_score: Decimal,
    pub stability_score: Decimal,
    pub confidence_memory: Decimal,
}

impl ConfidenceScore {
    pub fn calculate(
        edge_score: Decimal,
        expectancy_score: Decimal,
        sample_adequacy: Decimal,
        regime_score: Decimal,
        session_score: Decimal,
        symbol_score: Decimal,
        timeframe_score: Decimal,
        stability_score: Decimal,
        confidence_memory: Decimal,
        penalties: Vec<ConfidencePenalty>,
    ) -> Self {
        let raw_score = edge_score * dec!(0.2) +
                        expectancy_score * dec!(0.2) +
                        sample_adequacy * dec!(0.15) +
                        stability_score * dec!(0.15) +
                        regime_score * dec!(0.1) +
                        session_score * dec!(0.05) +
                        symbol_score * dec!(0.05) +
                        timeframe_score * dec!(0.05) +
                        confidence_memory * dec!(0.05);

        let mut total_penalty_impact = Decimal::ZERO;
        for penalty in &penalties {
            total_penalty_impact += penalty.impact;
        }

        let mut final_score = raw_score - total_penalty_impact;
        if final_score < Decimal::ZERO {
            final_score = Decimal::ZERO;
        } else if final_score > dec!(100.0) {
            final_score = dec!(100.0);
        }

        let level = if final_score < dec!(20.0) {
            ConfidenceLevel::VeryLow
        } else if final_score < dec!(40.0) {
            ConfidenceLevel::Low
        } else if final_score < dec!(60.0) {
            ConfidenceLevel::Moderate
        } else if final_score < dec!(80.0) {
            ConfidenceLevel::High
        } else {
            ConfidenceLevel::VeryHigh
        };

        Self {
            raw_score,
            final_score,
            level,
            penalties,
            edge_score,
            expectancy_score,
            sample_adequacy,
            regime_score,
            session_score,
            symbol_score,
            timeframe_score,
            stability_score,
            confidence_memory,
        }
    }
}
