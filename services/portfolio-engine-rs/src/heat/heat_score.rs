use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortfolioHeatState {
    Cold,     // 0-20
    Normal,   // 20-40
    Warm,     // 40-60
    Hot,      // 60-80
    Critical, // 80-95
    Frozen,   // 95-100
}

impl PortfolioHeatState {
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=20 => Self::Cold,
            21..=40 => Self::Normal,
            41..=60 => Self::Warm,
            61..=80 => Self::Hot,
            81..=95 => Self::Critical,
            _ => Self::Frozen,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactorContribution {
    pub name: String,
    pub weight: Decimal,
    pub contribution: u8,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeatContributionBreakdown {
    pub factors: Vec<FactorContribution>,
    pub total_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHeat {
    pub score: u8,
    pub state: PortfolioHeatState,
    pub breakdown: HeatContributionBreakdown,
}

impl PortfolioHeat {
    pub fn new(score: u8, breakdown: HeatContributionBreakdown) -> Self {
        let bounded_score = score.min(100);
        Self {
            score: bounded_score,
            state: PortfolioHeatState::from_score(bounded_score),
            breakdown,
        }
    }

    pub fn apply_decay(&mut self, amount: u8) {
        self.score = self.score.saturating_sub(amount);
        self.state = PortfolioHeatState::from_score(self.score);
    }
}
