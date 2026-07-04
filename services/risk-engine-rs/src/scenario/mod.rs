use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressScenario {
    pub id: String,
    pub name: String,
    pub equity_shock_percent: Decimal,
    pub volatility_multiplier: Decimal,
}

impl StressScenario {
    pub fn new(id: &str, name: &str, shock: Decimal, vol_mult: Decimal) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            equity_shock_percent: shock,
            volatility_multiplier: vol_mult,
        }
    }

    pub fn list_standard_scenarios() -> Vec<Self> {
        vec![
            Self::new("CHF_2015", "CHF Flash Depeg 2015", dec!(-0.15), dec!(3.5)),
            Self::new("COVID_2020", "COVID-19 Selloff 2020", dec!(-0.25), dec!(4.0)),
            Self::new("LEHMAN_2008", "Lehman Collapse 2008", dec!(-0.35), dec!(5.0)),
        ]
    }
}
