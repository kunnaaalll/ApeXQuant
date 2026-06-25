use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Topic {
    MarketTick,
    MarketCandle,
    MarketIntelligence,
    StrategySignal,
    StrategyHealth,
    StrategyRecommendation,
    RiskAlert,
    RiskLimit,
    RiskFreeze,
    PortfolioAllocation,
    PortfolioExposure,
    PortfolioDrawdown,
    ExecutionOrder,
    ExecutionFill,
    ExecutionRejection,
    SystemHealth,
    SystemCertification,
    SystemShadow,
    Custom(String),
}

impl Topic {
    pub fn as_str(&self) -> &str {
        match self {
            Topic::MarketTick => "market.tick",
            Topic::MarketCandle => "market.candle",
            Topic::MarketIntelligence => "market.intelligence",
            Topic::StrategySignal => "strategy.signal",
            Topic::StrategyHealth => "strategy.health",
            Topic::StrategyRecommendation => "strategy.recommendation",
            Topic::RiskAlert => "risk.alert",
            Topic::RiskLimit => "risk.limit",
            Topic::RiskFreeze => "risk.freeze",
            Topic::PortfolioAllocation => "portfolio.allocation",
            Topic::PortfolioExposure => "portfolio.exposure",
            Topic::PortfolioDrawdown => "portfolio.drawdown",
            Topic::ExecutionOrder => "execution.order",
            Topic::ExecutionFill => "execution.fill",
            Topic::ExecutionRejection => "execution.rejection",
            Topic::SystemHealth => "system.health",
            Topic::SystemCertification => "system.certification",
            Topic::SystemShadow => "system.shadow",
            Topic::Custom(s) => s.as_str(),
        }
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
