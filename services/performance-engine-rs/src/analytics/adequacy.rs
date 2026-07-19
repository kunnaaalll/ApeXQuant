use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdequacyState {
    Insufficient,
    LowConfidence,
    Reliable,
    InstitutionalGrade,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdequacyScore {
    pub score: Decimal,
    pub state: AdequacyState,
    pub confidence_interval: Decimal,
    pub overfitting_penalty: Decimal,
}

pub struct SampleAdequacyEngine;

impl SampleAdequacyEngine {
    pub const MIN_TRADES_INSUFFICIENT: u32 = 30;
    pub const MIN_TRADES_RELIABLE: u32 = 100;
    pub const MIN_TRADES_INSTITUTIONAL: u32 = 500;

    pub fn evaluate(trade_count: u32, win_rate: Decimal, stability: Decimal) -> AdequacyScore {
        let state = Self::determine_state(trade_count);
        let score = Self::calculate_score(trade_count, stability);
        let confidence_interval = Self::calculate_confidence_interval(trade_count, win_rate);
        let overfitting_penalty = Self::calculate_overfitting_penalty(trade_count);

        AdequacyScore {
            score,
            state,
            confidence_interval,
            overfitting_penalty,
        }
    }

    fn determine_state(trade_count: u32) -> AdequacyState {
        if trade_count < Self::MIN_TRADES_INSUFFICIENT {
            AdequacyState::Insufficient
        } else if trade_count < Self::MIN_TRADES_RELIABLE {
            AdequacyState::LowConfidence
        } else if trade_count < Self::MIN_TRADES_INSTITUTIONAL {
            AdequacyState::Reliable
        } else {
            AdequacyState::InstitutionalGrade
        }
    }

    fn calculate_score(trade_count: u32, stability: Decimal) -> Decimal {
        if trade_count == 0 {
            return Decimal::ZERO;
        }

        let log_scale = Decimal::from_f64(f64::ln(trade_count as f64)).unwrap_or(Decimal::ZERO);

        // Simple scoring based on log(count) * stability
        let max_score = Decimal::from_f64(10.0).unwrap_or_default();
        let score = log_scale * stability;

        if score > max_score {
            max_score
        } else if score < Decimal::ZERO {
            Decimal::ZERO
        } else {
            score
        }
    }

    fn calculate_confidence_interval(trade_count: u32, _win_rate: Decimal) -> Decimal {
        if trade_count == 0 {
            return Decimal::from_f64(100.0).unwrap_or_default(); // 100% uncertainty
        }
        // Approximate standard error of proportion: 1 / sqrt(N)
        let n_f64 = trade_count as f64;
        let se = 1.0 / n_f64.sqrt();
        // 95% CI multiplier ~1.96
        let ci = se * 1.96;
        Decimal::from_f64(ci).unwrap_or(Decimal::ONE)
    }

    fn calculate_overfitting_penalty(trade_count: u32) -> Decimal {
        if trade_count == 0 {
            return Decimal::ONE; // 100% penalty
        }

        if trade_count >= Self::MIN_TRADES_INSTITUTIONAL {
            return Decimal::ZERO; // No penalty for institutional grade
        }

        // Exponential decay of penalty as trade count grows
        let penalty = f64::exp(-(trade_count as f64) / 100.0);
        Decimal::from_f64(penalty).unwrap_or(Decimal::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adequacy_state() {
        assert_eq!(
            SampleAdequacyEngine::determine_state(10),
            AdequacyState::Insufficient
        );
        assert_eq!(
            SampleAdequacyEngine::determine_state(50),
            AdequacyState::LowConfidence
        );
        assert_eq!(
            SampleAdequacyEngine::determine_state(200),
            AdequacyState::Reliable
        );
        assert_eq!(
            SampleAdequacyEngine::determine_state(1000),
            AdequacyState::InstitutionalGrade
        );
    }
}
