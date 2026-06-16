use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Theme {
    RiskOn,
    RiskOff,
    Inflation,
    USD,
    Tech,
    Crypto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeExposure {
    pub theme: Theme,
    pub exposure: Decimal,
    pub dominance_score: Decimal,
}

impl ThemeExposure {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            exposure: Decimal::ZERO,
            dominance_score: Decimal::ZERO,
        }
    }

    pub fn calculate_clustering(
        exposures: &mut HashMap<Theme, ThemeExposure>,
        total_theme_exposure: Decimal,
    ) {
        if total_theme_exposure == Decimal::ZERO {
            return;
        }

        for theme_exp in exposures.values_mut() {
            theme_exp.dominance_score = (theme_exp.exposure / total_theme_exposure) * Decimal::from(100);
        }
    }
}
