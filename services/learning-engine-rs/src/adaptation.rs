use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdaptationState {
    Learning,
    Stable,
    Adapting,
    Recovering,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationMetrics {
    pub regime_adaptation: Decimal,
    pub volatility_adaptation: Decimal,
    pub session_adaptation: Decimal,
    pub symbol_adaptation: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationResult {
    pub strategy_id: String,
    pub state: AdaptationState,
    pub score: Decimal, // 0 to 100
    pub metrics: AdaptationMetrics,
}

impl AdaptationResult {
    pub fn new(strategy_id: String) -> Self {
        Self {
            strategy_id,
            state: AdaptationState::Learning,
            score: Decimal::new(50, 0), // Base score of 50
            metrics: AdaptationMetrics {
                regime_adaptation: Decimal::new(50, 0),
                volatility_adaptation: Decimal::new(50, 0),
                session_adaptation: Decimal::new(50, 0),
                symbol_adaptation: Decimal::new(50, 0),
            },
        }
    }

    pub fn with_metrics(strategy_id: String, metrics: AdaptationMetrics) -> Self {
        let mut result = Self {
            strategy_id,
            state: AdaptationState::Learning,
            score: Decimal::new(50, 0),
            metrics,
        };
        let _ = result.calculate_score();
        result
    }

    pub fn calculate_score(&mut self) -> Result<(), &'static str> {
        let four = Decimal::new(4, 0);
        
        let total = self.metrics.regime_adaptation 
                  + self.metrics.volatility_adaptation 
                  + self.metrics.session_adaptation 
                  + self.metrics.symbol_adaptation;
        
        // Simple average for the score
        self.score = total / four;
        
        let hundred = Decimal::new(100, 0);
        let seventy = Decimal::new(70, 0);
        let forty = Decimal::new(40, 0);
        let twenty = Decimal::new(20, 0);

        if self.score > hundred {
            self.score = hundred;
        }

        if self.score >= seventy {
            self.state = AdaptationState::Stable;
        } else if self.score >= forty {
            self.state = AdaptationState::Adapting;
        } else if self.score >= twenty {
            self.state = AdaptationState::Recovering;
        } else {
            self.state = AdaptationState::Failed;
        }

        Ok(())
    }
}
