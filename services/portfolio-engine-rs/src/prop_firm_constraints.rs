use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropFirmRules {
    pub max_daily_loss: Decimal,
    pub max_total_loss: Decimal,
    pub max_position_size: Decimal,
    pub weekend_holding_allowed: bool,
    pub news_trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule: String,
    pub description: String,
}

pub struct PropFirmConstraintEngine;

impl Default for PropFirmConstraintEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PropFirmConstraintEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_trade(
        &self,
        rules: &PropFirmRules,
        position_size: Decimal,
        is_weekend: bool,
        is_news_event: bool,
    ) -> Vec<RuleViolation> {
        let mut violations = Vec::new();

        if position_size > rules.max_position_size {
            violations.push(RuleViolation {
                rule: "MAX_POSITION_SIZE".to_string(),
                description: format!("Position size {} exceeds maximum allowed {}", position_size, rules.max_position_size),
            });
        }

        if is_weekend && !rules.weekend_holding_allowed {
            violations.push(RuleViolation {
                rule: "WEEKEND_HOLDING".to_string(),
                description: "Holding positions over the weekend is not allowed".to_string(),
            });
        }

        if is_news_event && !rules.news_trading_allowed {
            violations.push(RuleViolation {
                rule: "NEWS_TRADING".to_string(),
                description: "Trading during high-impact news events is not allowed".to_string(),
            });
        }

        violations
    }

    pub fn validate_drawdown(
        &self,
        rules: &PropFirmRules,
        daily_loss: Decimal,
        total_loss: Decimal,
    ) -> Vec<RuleViolation> {
        let mut violations = Vec::new();

        if daily_loss > rules.max_daily_loss {
            violations.push(RuleViolation {
                rule: "MAX_DAILY_LOSS".to_string(),
                description: format!("Daily loss {} exceeds maximum allowed {}", daily_loss, rules.max_daily_loss),
            });
        }

        if total_loss > rules.max_total_loss {
            violations.push(RuleViolation {
                rule: "MAX_TOTAL_LOSS".to_string(),
                description: format!("Total loss {} exceeds maximum allowed {}", total_loss, rules.max_total_loss),
            });
        }

        violations
    }
}
