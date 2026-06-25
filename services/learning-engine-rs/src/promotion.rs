use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PromotionLevel {
    Experimental,
    Shadow,
    Candidate,
    Approved,
    Production,
    Elite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRequirements {
    pub min_sample_size: u64,
    pub min_confidence: Decimal,
    pub max_drawdown: Decimal, // Max drawdown allowed (e.g. 10.0 for 10%)
    pub regime_robustness_score: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub sample_size: u64,
    pub confidence: Decimal,
    pub current_drawdown: Decimal,
    pub regime_robustness: Decimal,
}

#[derive(Debug, Clone)]
pub struct PromotionLadder {
    requirements: HashMap<PromotionLevel, PromotionRequirements>,
}

impl Default for PromotionLadder {
    fn default() -> Self {
        Self::new()
    }
}

impl PromotionLadder {
    pub fn new() -> Self {
        let mut reqs = HashMap::new();
        // Setup initial default institutional thresholds
        reqs.insert(PromotionLevel::Shadow, PromotionRequirements {
            min_sample_size: 100,
            min_confidence: Decimal::new(50, 0),
            max_drawdown: Decimal::new(15, 0),
            regime_robustness_score: Decimal::new(40, 0),
        });
        reqs.insert(PromotionLevel::Candidate, PromotionRequirements {
            min_sample_size: 500,
            min_confidence: Decimal::new(70, 0),
            max_drawdown: Decimal::new(10, 0),
            regime_robustness_score: Decimal::new(60, 0),
        });
        reqs.insert(PromotionLevel::Approved, PromotionRequirements {
            min_sample_size: 1000,
            min_confidence: Decimal::new(80, 0),
            max_drawdown: Decimal::new(8, 0),
            regime_robustness_score: Decimal::new(75, 0),
        });
        reqs.insert(PromotionLevel::Production, PromotionRequirements {
            min_sample_size: 5000,
            min_confidence: Decimal::new(90, 0),
            max_drawdown: Decimal::new(5, 0),
            regime_robustness_score: Decimal::new(85, 0),
        });
        reqs.insert(PromotionLevel::Elite, PromotionRequirements {
            min_sample_size: 20000,
            min_confidence: Decimal::new(95, 0),
            max_drawdown: Decimal::new(3, 0),
            regime_robustness_score: Decimal::new(95, 0),
        });
        Self { requirements: reqs }
    }

    pub fn evaluate_promotion(
        &self,
        current_level: PromotionLevel,
        perf: &StrategyPerformance,
    ) -> Result<Option<PromotionLevel>, &'static str> {
        let next_level = match current_level {
            PromotionLevel::Experimental => PromotionLevel::Shadow,
            PromotionLevel::Shadow => PromotionLevel::Candidate,
            PromotionLevel::Candidate => PromotionLevel::Approved,
            PromotionLevel::Approved => PromotionLevel::Production,
            PromotionLevel::Production => PromotionLevel::Elite,
            PromotionLevel::Elite => return Ok(None), // Already at top
        };

        let reqs = self.requirements.get(&next_level).ok_or("Missing requirements for level")?;

        if perf.sample_size >= reqs.min_sample_size
            && perf.confidence >= reqs.min_confidence
            && perf.current_drawdown <= reqs.max_drawdown
            && perf.regime_robustness >= reqs.regime_robustness_score
        {
            Ok(Some(next_level))
        } else {
            Ok(None)
        }
    }
}
