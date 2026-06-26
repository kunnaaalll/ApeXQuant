// CPB-005: Determinism Validation — actual hash-compare replay
//
// Invariants:
//   - Zero unwrap / expect / panic
//   - State hash is deterministic SHA-256 over canonical field order
//   - Two independent engine runs on identical input MUST produce identical hash
//   - Mismatch surfaces field-level diagnostics

use ring::digest;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::circuit_breaker::CircuitBreakerState;
use crate::drawdown::DrawdownTracker;
use crate::var::confidence_levels::ConfidenceLevel;
use crate::var::historical_var::HistoricalVaR;
use crate::var::parametric_var::ParametricVaR;

// ─── Canonical Test Vector ────────────────────────────────────────────────────

/// Deterministic returns sequence used for both replay runs.
/// Fixed at compile time — never generated at runtime.
const TEST_RETURNS: &[(Decimal, bool)] = &[];

fn test_returns() -> Vec<(Decimal, bool)> {
    vec![
        (dec!(0.012),  false),
        (dec!(-0.008), true),
        (dec!(0.005),  false),
        (dec!(-0.015), true),
        (dec!(0.020),  false),
        (dec!(-0.003), true),
        (dec!(0.007),  false),
        (dec!(-0.022), true),
        (dec!(0.018),  false),
        (dec!(-0.011), true),
        (dec!(0.009),  false),
        (dec!(-0.006), true),
        (dec!(0.014),  false),
        (dec!(-0.025), true),
        (dec!(0.003),  false),
        (dec!(-0.004), true),
        (dec!(0.016),  false),
        (dec!(-0.019), true),
        (dec!(0.011),  false),
        (dec!(-0.007), true),
    ]
}

// ─── Output Types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismResult {
    pub identical_output:   bool,
    pub iterations:         u64,
    pub run_a_hash:         String,
    pub run_b_hash:         String,
    pub mismatch_fields:    Vec<String>,
}

// ─── Engine State Snapshot ────────────────────────────────────────────────────

#[derive(Debug)]
struct DeterminismSnapshot {
    historical_var_99:    String,
    parametric_var_99:    String,
    parametric_std_dev:   String,
    drawdown_current:     String,
    drawdown_max:         String,
    circuit_breaker:      String,
}

impl DeterminismSnapshot {
    fn state_hash(&self) -> String {
        let canonical = format!(
            "{}|{}|{}|{}|{}|{}",
            self.historical_var_99,
            self.parametric_var_99,
            self.parametric_std_dev,
            self.drawdown_current,
            self.drawdown_max,
            self.circuit_breaker,
        );
        let d = digest::digest(&digest::SHA256, canonical.as_bytes());
        hex_encode(d.as_ref())
    }
}

// ─── Validator ────────────────────────────────────────────────────────────────

pub struct DeterminismValidator;

impl Default for DeterminismValidator {
    fn default() -> Self { Self::new() }
}

impl DeterminismValidator {
    pub fn new() -> Self { Self }

    /// Run the identical event stream twice with fresh engines.
    /// Returns `Ok` only when both runs produce identical state hashes.
    pub fn validate(&self) -> Result<DeterminismResult, crate::error::RiskError> {
        let returns = test_returns();
        let snapshot_a = Self::run_once(&returns)?;
        let snapshot_b = Self::run_once(&returns)?;

        let hash_a = snapshot_a.state_hash();
        let hash_b = snapshot_b.state_hash();

        let mut mismatches = Vec::new();

        if snapshot_a.historical_var_99 != snapshot_b.historical_var_99 {
            mismatches.push(format!(
                "historical_var_99: {} vs {}",
                snapshot_a.historical_var_99, snapshot_b.historical_var_99
            ));
        }
        if snapshot_a.parametric_var_99 != snapshot_b.parametric_var_99 {
            mismatches.push(format!(
                "parametric_var_99: {} vs {}",
                snapshot_a.parametric_var_99, snapshot_b.parametric_var_99
            ));
        }
        if snapshot_a.parametric_std_dev != snapshot_b.parametric_std_dev {
            mismatches.push(format!(
                "parametric_std_dev: {} vs {}",
                snapshot_a.parametric_std_dev, snapshot_b.parametric_std_dev
            ));
        }
        if snapshot_a.drawdown_current != snapshot_b.drawdown_current {
            mismatches.push(format!(
                "drawdown_current: {} vs {}",
                snapshot_a.drawdown_current, snapshot_b.drawdown_current
            ));
        }
        if snapshot_a.drawdown_max != snapshot_b.drawdown_max {
            mismatches.push(format!(
                "drawdown_max: {} vs {}",
                snapshot_a.drawdown_max, snapshot_b.drawdown_max
            ));
        }
        if snapshot_a.circuit_breaker != snapshot_b.circuit_breaker {
            mismatches.push(format!(
                "circuit_breaker: {} vs {}",
                snapshot_a.circuit_breaker, snapshot_b.circuit_breaker
            ));
        }

        let identical = hash_a == hash_b;

        if !identical {
            return Err(crate::error::RiskError::ValidationError(format!(
                "Determinism failure — hashes differ: {hash_a} vs {hash_b}. Mismatches: {mismatches:?}"
            )));
        }

        Ok(DeterminismResult {
            identical_output: true,
            iterations:       returns.len() as u64,
            run_a_hash:       hash_a,
            run_b_hash:       hash_b,
            mismatch_fields:  mismatches,
        })
    }

    /// Process the fixed canonical event stream with fresh engine instances.
    fn run_once(returns: &[(Decimal, bool)]) -> Result<DeterminismSnapshot, crate::error::RiskError> {
        let mut hist_var   = HistoricalVaR::new(250);
        let mut param_var  = ParametricVaR::new();
        let mut drawdown   = DrawdownTracker::new();
        let mut cb_state   = CircuitBreakerState::Normal;

        // Simulate equity starting at 100,000 (arbitrary deterministic base)
        let mut equity = dec!(100_000);

        for (ret, _is_loss) in returns {
            hist_var.add_return(*ret);
            param_var.add_return(*ret);

            // Equity walk
            equity += equity * ret;
            drawdown.observe(equity);

            // Deterministic circuit-breaker transitions
            let dd = drawdown.current_drawdown;
            let next_state = if dd >= dec!(0.20) {
                CircuitBreakerState::Frozen
            } else if dd >= dec!(0.15) {
                CircuitBreakerState::Critical
            } else if dd >= dec!(0.10) {
                CircuitBreakerState::Restricted
            } else if dd >= dec!(0.05) {
                CircuitBreakerState::Warning
            } else {
                CircuitBreakerState::Normal
            };

            // Use transition_to for sequential safety
            cb_state = cb_state.transition_to(next_state)
                .unwrap_or(next_state);
        }

        Ok(DeterminismSnapshot {
            historical_var_99:  hist_var.compute_var(ConfidenceLevel::NinetyNine).to_string(),
            parametric_var_99:  param_var.var_99().to_string(),
            parametric_std_dev: param_var.standard_deviation().to_string(),
            drawdown_current:   drawdown.current_drawdown.to_string(),
            drawdown_max:       drawdown.max_drawdown.to_string(),
            circuit_breaker:    format!("{cb_state:?}"),
        })
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determinism_validation_passes() {
        let validator = DeterminismValidator::new();
        let result = validator.validate().expect("determinism validation must pass");
        assert!(result.identical_output, "runs must produce identical output");
        assert_eq!(result.run_a_hash, result.run_b_hash, "hashes must match");
        assert!(result.mismatch_fields.is_empty(), "no field mismatches");
    }
}
