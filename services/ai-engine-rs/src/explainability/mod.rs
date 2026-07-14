use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureImportance {
    pub feature_name: String,
    pub weight: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionExplanation {
    pub prediction_id: Uuid,
    pub target_symbol: String,
    pub top_features: Vec<FeatureImportance>,
    pub confidence_breakdown: Vec<(String, Decimal)>,
    pub historical_similarity_score: Decimal,
    pub regime_match: String,
    pub risk_factors: Vec<String>,
    pub decision_summary: String,
    pub alternative_actions: Vec<String>,
}

pub struct ExplainabilityEngine;

impl ExplainabilityEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn explain_prediction(
        &self,
        prediction_id: Uuid,
        symbol: &str,
        features: &[FeatureImportance],
        base_confidence: Decimal,
        regime: &str,
    ) -> PredictionExplanation {
        // Deterministic generation of explanations
        let mut top_features = features.to_vec();
        top_features.sort_by(|a, b| b.weight.cmp(&a.weight));
        top_features.truncate(5); // Top 5 features

        let historical_similarity = if base_confidence > Decimal::new(70, 2) {
            Decimal::new(85, 2) // High confidence implies strong historical precedent
        } else {
            Decimal::new(45, 2)
        };

        let risk_factors = if regime == "HighVolatility" || regime == "LiquidityCrisis" {
            vec!["High slippage expected".to_string(), "Wider spreads".to_string()]
        } else {
            vec!["Standard market risk".to_string()]
        };

        PredictionExplanation {
            prediction_id,
            target_symbol: symbol.to_string(),
            top_features: top_features.clone(),
            confidence_breakdown: vec![
                ("Base Model".to_string(), base_confidence),
                ("Regime Adjustment".to_string(), Decimal::new(5, 2)),
                ("Volatility Penalty".to_string(), Decimal::new(-2, 2)),
            ],
            historical_similarity_score: historical_similarity,
            regime_match: regime.to_string(),
            risk_factors,
            decision_summary: format!("Prediction driven by {} in {} regime.", top_features.first().map(|f| f.feature_name.as_str()).unwrap_or("unknown features"), regime),
            alternative_actions: vec!["Scale down position size by 50%".to_string()],
        }
    }
}
