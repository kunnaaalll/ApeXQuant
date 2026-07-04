use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Conservative,
    Moderate,
    Aggressive,
}

#[derive(Debug, Clone)]
pub struct RiskProfile {
    pub level: RiskLevel,
    pub max_leverage: Decimal,
    pub max_drawdown: Decimal,
    pub sizing_multiplier: Decimal,
}

impl RiskProfile {
    pub fn for_level(level: RiskLevel) -> Self {
        match level {
            RiskLevel::Conservative => Self {
                level,
                max_leverage: dec!(3.0),
                max_drawdown: dec!(0.05),
                sizing_multiplier: dec!(0.5),
            },
            RiskLevel::Moderate => Self {
                level,
                max_leverage: dec!(5.0),
                max_drawdown: dec!(0.10),
                sizing_multiplier: dec!(1.0),
            },
            RiskLevel::Aggressive => Self {
                level,
                max_leverage: dec!(10.0),
                max_drawdown: dec!(0.20),
                sizing_multiplier: dec!(1.5),
            },
        }
    }
}
