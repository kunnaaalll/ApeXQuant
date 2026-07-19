pub mod correlation;
pub mod events;
pub mod leverage;
pub mod liquidity;
pub mod scenarios;
pub mod severity;
pub mod snapshot;
pub mod survival;
pub mod volatility;

#[cfg(test)]
mod tests;

pub use scenarios::HistoricalScenario;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

// ─── Stress Result ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressScenarioResult {
    pub scenario_id: String,
    pub estimated_loss: Decimal,
    pub survived: bool,
}

// ─── Stress Engine Facade ─────────────────────────────────────────────────────

/// Deterministic stress test runner.
/// Maps scenario IDs (string) to canonical `HistoricalScenario` multipliers
/// and applies them to a gross_exposure reference.
#[derive(Debug, Clone)]
pub struct StressEngine {
    /// Current gross exposure used as the base for loss estimates.
    pub gross_exposure: Decimal,
    /// Maximum tolerable loss fraction before survival fails.
    pub loss_tolerance: Decimal,
}

impl StressEngine {
    pub fn new() -> Self {
        Self {
            gross_exposure: Decimal::ZERO,
            loss_tolerance: dec!(0.20), // 20% max tolerable loss
        }
    }

    pub fn set_exposure(&mut self, exposure: Decimal) {
        self.gross_exposure = exposure;
    }

    /// Run a named scenario and return a deterministic result.
    /// Unknown scenario IDs use a conservative 2× multiplier.
    pub fn run_scenario(&self, scenario_id: &str) -> StressScenarioResult {
        let vol_mult = match scenario_id {
            "flash_crash" => dec!(3.0),
            "black_monday_1987" => dec!(5.0),
            "covid_crash_2020" => dec!(4.5),
            "dot_com_bubble" => dec!(2.5),
            "lehman_2008" => dec!(4.0),
            "synthetic_extreme" => dec!(6.0),
            _ => dec!(2.0), // conservative default
        };

        // Estimated loss = gross_exposure × (vol_mult − 1) × 0.05 base factor
        let estimated_loss = self.gross_exposure * (vol_mult - Decimal::from(1)) * dec!(0.05);
        let survived = estimated_loss < (self.gross_exposure * self.loss_tolerance);

        StressScenarioResult {
            scenario_id: scenario_id.to_owned(),
            estimated_loss,
            survived,
        }
    }
}

impl Default for StressEngine {
    fn default() -> Self {
        Self::new()
    }
}
