use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::exposure::concentration::ConcentrationLevel;
use crate::exposure::currency_exposure::CurrencyExposure;
use crate::exposure::sector_exposure::SectorExposure;
use crate::exposure::symbol_exposure::SymbolExposure;
use crate::exposure::theme_exposure::ThemeExposure;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskState {
    Normal,
    Elevated,
    High,
    Critical,
    Frozen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureRiskState {
    pub gross_exposure: Decimal,
    pub net_exposure: Decimal,
    pub symbol_exposures: std::collections::HashMap<String, SymbolExposure>,
    pub currency_exposures: std::collections::HashMap<String, CurrencyExposure>,
    pub sector_exposures: std::collections::HashMap<String, SectorExposure>,
    pub theme_exposures: std::collections::HashMap<String, ThemeExposure>,
    pub concentration_score: Decimal,
    pub diversification_score: Decimal,
    pub largest_position_weight: Decimal,
    pub largest_cluster_weight: Decimal,
    pub state: RiskState,
}

impl ExposureRiskState {
    pub fn new() -> Self {
        Self {
            gross_exposure: Decimal::ZERO,
            net_exposure: Decimal::ZERO,
            symbol_exposures: std::collections::HashMap::new(),
            currency_exposures: std::collections::HashMap::new(),
            sector_exposures: std::collections::HashMap::new(),
            theme_exposures: std::collections::HashMap::new(),
            concentration_score: Decimal::ZERO,
            diversification_score: Decimal::from(100),
            largest_position_weight: Decimal::ZERO,
            largest_cluster_weight: Decimal::ZERO,
            state: RiskState::Normal,
        }
    }

    pub fn determine_state(&mut self, concentration_level: ConcentrationLevel) {
        // Deterministic state transition
        match concentration_level {
            ConcentrationLevel::Normal => {
                if self.state != RiskState::Frozen {
                    self.state = RiskState::Normal;
                }
            }
            ConcentrationLevel::Elevated => {
                if self.state == RiskState::Normal {
                    self.state = RiskState::Elevated;
                }
            }
            ConcentrationLevel::High => {
                if self.state == RiskState::Normal || self.state == RiskState::Elevated {
                    self.state = RiskState::High;
                }
            }
            ConcentrationLevel::Critical => {
                if self.state != RiskState::Frozen {
                    self.state = RiskState::Critical;
                }
            }
            ConcentrationLevel::Collapse => {
                self.state = RiskState::Frozen;
            }
        }
    }
}

impl Default for ExposureRiskState {
    fn default() -> Self {
        Self::new()
    }
}
