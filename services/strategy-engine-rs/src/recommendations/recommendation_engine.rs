use super::ReasonCode;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationAction {
    Increase,
    Maintain,
    Reduce,
    Pause,
    Research,
    Retire,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recommendation {
    pub action: RecommendationAction,
    pub confidence: Decimal,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendationEngine {
    current_recommendation: Recommendation,
}

impl RecommendationEngine {
    pub fn new() -> Self {
        Self {
            current_recommendation: Recommendation {
                action: RecommendationAction::Research,
                confidence: dec!(0.0),
                reason_codes: vec![],
            },
        }
    }

    pub fn generate(
        &mut self,
        edge_strength: Decimal,
        risk_level: Decimal,
        stability: Decimal,
    ) -> &Recommendation {
        let mut codes = Vec::new();

        let action = if risk_level > dec!(0.8) {
            codes.push(ReasonCode::ExcessiveRisk);
            RecommendationAction::Retire
        } else if risk_level > dec!(0.5) || edge_strength < dec!(-0.2) {
            codes.push(ReasonCode::EdgeCollapsing);
            RecommendationAction::Pause
        } else if edge_strength > dec!(0.5) && stability > dec!(0.7) {
            codes.push(ReasonCode::EdgeEmerging);
            codes.push(ReasonCode::ExcellentStability);
            RecommendationAction::Increase
        } else if edge_strength > dec!(0.2) {
            codes.push(ReasonCode::StrongMomentum);
            RecommendationAction::Maintain
        } else {
            codes.push(ReasonCode::PoorSampleQuality);
            RecommendationAction::Research
        };

        // Determine confidence (0 - 100 bounded)
        let confidence_raw = (edge_strength.abs() * dec!(50.0)) + (stability * dec!(50.0));
        let confidence = if confidence_raw > dec!(100.0) {
            dec!(100.0)
        } else if confidence_raw < dec!(0.0) {
            dec!(0.0)
        } else {
            confidence_raw
        };

        self.current_recommendation = Recommendation {
            action,
            confidence,
            reason_codes: codes,
        };

        &self.current_recommendation
    }

    pub fn current(&self) -> &Recommendation {
        &self.current_recommendation
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}
