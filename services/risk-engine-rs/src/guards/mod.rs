//! Risk guards - final validation layer
//!
//! Purpose: Prevent impossible outputs, verify consistency, enforce constraints.
//! This is the last line of defense before risk assessments are returned.
use rust_decimal::prelude::FromPrimitive;

use crate::{
    DailyLimitState, DrawdownState, ExposureAnalysis, PositionSizeResult, RiskError, RiskInputs,
    RiskProfile,
};
use rust_decimal::Decimal;

pub mod consistency;
pub mod sanity;
pub mod validation;

pub use consistency::ConsistencyGuard;
pub use sanity::SanityGuard;
pub use validation::ValidationGuard;

/// Primary risk guard that orchestrates all validation
pub struct RiskGuard {
    validation: ValidationGuard,
    consistency: ConsistencyGuard,
    sanity: SanityGuard,
}

impl RiskGuard {
    /// Create new risk guard
    pub fn new(
        drawdown: std::sync::Arc<crate::DrawdownEngine>,
        daily_limits: std::sync::Arc<crate::DailyLimitsEngine>,
    ) -> Self {
        Self {
            validation: ValidationGuard::new(),
            consistency: ConsistencyGuard::new(),
            sanity: SanityGuard::new(),
        }
    }

    /// Validate a complete position sizing result
    pub fn validate(
        &self,
        result: &PositionSizeResult,
        exposure: &crate::exposure::ExposureMetrics,
        drawdown: &DrawdownState,
        daily: &DailyLimitState,
    ) -> Result<(), RiskError> {
        // 1. Validation - check bounds and types
        self.validation.validate_result(result)?;

        // 2. Consistency - internal logic checks
        self.consistency.check_result(result)?;

        // 3. Sanity - catch obvious errors
        self.sanity.check_sanity(result)?;

        // 4. State-based validation
        self.validate_state(result, drawdown, daily)?;

        // 5. Exposure validation
        self.validate_exposure(result, exposure)?;

        Ok(())
    }

    /// Quick validation for emergency checks
    pub fn quick_validate(&self, inputs: &RiskInputs) -> Result<(), RiskError> {
        self.validation.validate_inputs(inputs)
    }

    fn validate_state(
        &self,
        result: &PositionSizeResult,
        drawdown: &DrawdownState,
        daily: &DailyLimitState,
    ) -> Result<(), RiskError> {
        // Check drawdown states that block trading
        if drawdown.blocks_new_positions() && result.lot_size > Decimal::ZERO {
            return Err(RiskError::DrawdownLimitExceeded {
                current: "hard_limit".to_string(),
                limit: "trading_blocked".to_string(),
            });
        }

        // Check daily limit
        if daily.blocks_new_positions() && result.lot_size > Decimal::ZERO {
            return Err(RiskError::DailyLossLimitExceeded {
                loss: "limit".to_string(),
                limit: "reached".to_string(),
            });
        }

        Ok(())
    }

    fn validate_exposure(
        &self,
        result: &PositionSizeResult,
        exposure: &crate::exposure::ExposureMetrics,
    ) -> Result<(), RiskError> {
        // Check if we're exceeding max positions
        if exposure.total_positions > 10 && result.lot_size > Decimal::ZERO {
            return Err(RiskError::MaxPositionsReached(exposure.total_positions as u32));
        }

        // Check exposure limits
        if result.capital_at_risk > exposure.total_exposure * Decimal::from_f64(0.5).unwrap() {
            // Warning but not error - the position is large
            // In production, this might be configurable
        }

        Ok(())
    }
}

impl Default for RiskGuard {
    fn default() -> Self {
        Self::new(
            std::sync::Arc::new(crate::DrawdownEngine::default()),
            std::sync::Arc::new(crate::DailyLimitsEngine::default()),
        )
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// All checks passed
    Pass,
    /// Warning issued but not blocking
    Warning(String),
    /// Validation failed
    Fail(String),
}

impl ValidationResult {
    /// Convert to Result
    pub fn into_result(self) -> Result<(), RiskError> {
        match self {
            ValidationResult::Pass => Ok(()),
            ValidationResult::Warning(_) => Ok(()),
            ValidationResult::Fail(msg) => Err(RiskError::Validation(msg)),
        }
    }

    /// Check if validation passed
    pub fn is_pass(&self) -> bool {
        matches!(self, ValidationResult::Pass | ValidationResult::Warning(_))
    }

    /// Check if validation failed
    pub fn is_fail(&self) -> bool {
        matches!(self, ValidationResult::Fail(_))
    }
}
