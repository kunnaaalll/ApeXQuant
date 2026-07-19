use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConcentrationLevel {
    Normal,
    Elevated,
    High,
    Critical,
    Collapse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationMetrics {
    pub largest_position_pct: Decimal,
    pub largest_sector_pct: Decimal,
    pub largest_theme_pct: Decimal,
    pub largest_currency_pct: Decimal,
    pub concentration_score: Decimal,
    pub diversification_score: Decimal,
    pub level: ConcentrationLevel,
}

impl ConcentrationMetrics {
    pub fn new() -> Self {
        Self {
            largest_position_pct: Decimal::ZERO,
            largest_sector_pct: Decimal::ZERO,
            largest_theme_pct: Decimal::ZERO,
            largest_currency_pct: Decimal::ZERO,
            concentration_score: Decimal::ZERO,
            diversification_score: Decimal::from(100),
            level: ConcentrationLevel::Normal,
        }
    }

    pub fn calculate_scores(&mut self) {
        // Simple heuristic for score calculation
        let raw_concentration = (self.largest_position_pct * Decimal::from(2))
            + self.largest_sector_pct
            + self.largest_theme_pct
            + self.largest_currency_pct;

        let normalized = raw_concentration / Decimal::from(5);

        self.concentration_score = normalized.clamp(Decimal::ZERO, Decimal::from(100));
        self.diversification_score = (Decimal::from(100) - self.concentration_score)
            .clamp(Decimal::ZERO, Decimal::from(100));

        self.level = match self.concentration_score {
            s if s < Decimal::from(30) => ConcentrationLevel::Normal,
            s if s < Decimal::from(50) => ConcentrationLevel::Elevated,
            s if s < Decimal::from(70) => ConcentrationLevel::High,
            s if s < Decimal::from(90) => ConcentrationLevel::Critical,
            _ => ConcentrationLevel::Collapse,
        };
    }
}

impl Default for ConcentrationMetrics {
    fn default() -> Self {
        Self::new()
    }
}
