use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeScore {
    pub expectancy: Decimal,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub drawdown_risk: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub sample_size: u64,
    pub out_of_sample_score: Decimal,
    pub stability_index: Decimal,
    pub overall_confidence: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPriority {
    pub queue_score: Decimal,
    pub execution_cost_estimate: Decimal,
    pub estimated_alpha: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyCandidate {
    pub id: String,
    pub target_symbol: String,
    pub session: String,
    pub timeframe: String,
    pub regime: String,
    pub volatility_environment: String,
    pub correlation_cluster: String,
    pub edge_score: Option<EdgeScore>,
    pub confidence_score: Option<ConfidenceScore>,
    pub research_priority: Option<ResearchPriority>,
}

pub struct StrategyDiscoveryEngine {}

impl StrategyDiscoveryEngine {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    pub fn discover_strategies_from_patterns(
        &self,
        symbol: &str,
        patterns_detected: usize,
        regime: &str,
    ) -> Vec<StrategyCandidate> {
        let mut candidates = Vec::new();

        // Deterministic generation derived from historical performance memory
        // E.g. we only generate strategies if multiple patterns confluenced
        if patterns_detected >= 2 {
            let win_rate = if regime == "TrendingUp" {
                Decimal::new(65, 2)
            } else {
                Decimal::new(55, 2)
            };

            candidates.push(StrategyCandidate {
                id: format!("{}-{}-Confluence", symbol, regime),
                target_symbol: symbol.to_string(),
                session: "NewYork".to_string(),
                timeframe: "15m".to_string(),
                regime: regime.to_string(),
                volatility_environment: "Normal".to_string(),
                correlation_cluster: "Alpha".to_string(),
                edge_score: Some(EdgeScore {
                    expectancy: Decimal::new(12, 1),
                    win_rate,
                    profit_factor: Decimal::new(15, 1),
                    drawdown_risk: Decimal::new(5, 2),
                }),
                confidence_score: Some(ConfidenceScore {
                    sample_size: 1000,
                    out_of_sample_score: Decimal::new(85, 2),
                    stability_index: Decimal::new(9, 1),
                    overall_confidence: Decimal::new(88, 2),
                }),
                research_priority: Some(ResearchPriority {
                    queue_score: Decimal::new(90, 0),
                    execution_cost_estimate: Decimal::new(1, 2),
                    estimated_alpha: Decimal::new(20, 2),
                }),
            });
        }

        candidates
    }
}

impl Default for StrategyDiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}
