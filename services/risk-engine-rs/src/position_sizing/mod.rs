//! Position sizing strategies
use rust_decimal::prelude::FromPrimitive;

use crate::{
    DailyLimitState, DrawdownState, ExposureMetrics, RiskError, RiskInputs, RiskProfile,
    StreakState,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

mod fixed_fractional;
mod kelly;
mod volatility_adjusted;

pub use fixed_fractional::FixedFractionalSizing;
pub use kelly::KellySizing;
pub use volatility_adjusted::VolatilityAdjustedSizing;

/// Result of position size calculation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionSizeResult {
    /// Calculated lot size
    pub lot_size: Decimal,
    /// Risk percentage of capital
    pub risk_percent: Decimal,
    /// Capital at risk
    pub capital_at_risk: Decimal,
    /// Sizing method used
    pub method: SizingMethod,
    /// Explanation
    pub reasoning: String,
}

/// Sizing methods available
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SizingMethod {
    /// Fixed fractional position sizing
    FixedFractional,
    /// Full Kelly criterion
    KellyFull,
    /// Fractional Kelly (conservative)
    KellyFractional,
    /// Volatility-adjusted sizing
    VolatilityAdjusted,
    /// Confidence-adjusted sizing
    ConfidenceAdjusted,
    /// Conservative (reduced size)
    Conservative,
    /// Minimal sizing (circuit breaker, drawdown)
    Minimal,
}

/// Position sizing engine
pub struct PositionSizingEngine {
    kelly_fraction: Decimal,
    fixed: FixedFractionalSizing,
    kelly: KellySizing,
    volatility: VolatilityAdjustedSizing,
}

impl PositionSizingEngine {
    /// Create new position sizing engine
    pub fn new(kelly_fraction: Decimal) -> Self {
        Self {
            kelly_fraction,
            fixed: FixedFractionalSizing::default(),
            kelly: KellySizing::new(kelly_fraction),
            volatility: VolatilityAdjustedSizing::default(),
        }
    }

    /// Calculate position size given all inputs
    #[allow(clippy::too_many_arguments)]
    pub fn calculate(
        &self,
        inputs: &RiskInputs,
        profile: &RiskProfile,
        drawdown: &DrawdownState,
        daily: &DailyLimitState,
        streak: &StreakState,
        exposure: &ExposureMetrics,
        correlation: &Decimal,
        vol_metrics: &crate::volatility::VolatilityMetrics,
    ) -> Result<PositionSizeResult, RiskError> {
        // Base calculation using fixed fractional
        let mut result = self.fixed.calculate(inputs)?;

        // Apply Kelly adjustment if conditions are favorable
        if inputs.confluence_score > Decimal::from(7) && streak.win_rate > Decimal::from_f64(0.5).unwrap_or(Decimal::ONE) {
            let kelly_result = self.kelly.calculate(inputs, streak);
            if kelly_result.lot_size < result.lot_size {
                result = kelly_result;
            }
        }

        // Apply volatility adjustment
        result = self.volatility.adjust(result, vol_metrics, inputs);

        // Apply profile-based scaling
        result = self.apply_profile_scaling(result, profile);

        // Apply drawdown scaling
        result = self.apply_drawdown_scaling(result, drawdown);

        // Apply daily limit scaling
        result = self.apply_daily_scaling(result, daily);

        // Apply streak adjustment
        result = self.apply_streak_adjustment(result, streak);

        // Apply correlation reduction
        result = self.apply_correlation_reduction(result, correlation);

        // Apply exposure cap
        result = self.apply_exposure_cap(result, exposure, inputs.equity);

        // Ensure minimum meaningful size
        if result.lot_size > Decimal::ZERO && result.lot_size < Decimal::from_f64(0.01).unwrap_or(Decimal::ONE) {
            result.lot_size = Decimal::from_f64(0.01).unwrap_or(Decimal::ONE);
        }

        Ok(result)
    }

    fn apply_profile_scaling(&self, mut result: PositionSizeResult, profile: &RiskProfile) -> PositionSizeResult {
        let multiplier = match profile {
            RiskProfile::VeryConservative => Decimal::from_f64(0.5).unwrap_or(Decimal::ONE),
            RiskProfile::Conservative => Decimal::from_f64(0.75).unwrap_or(Decimal::ONE),
            RiskProfile::Normal => Decimal::ONE,
            RiskProfile::Aggressive => Decimal::from_f64(1.25).unwrap_or(Decimal::ONE),
            RiskProfile::HighConviction => Decimal::from_f64(1.5).unwrap_or(Decimal::ONE),
        };

        result.lot_size = result.lot_size * multiplier;
        result.risk_percent = result.risk_percent * multiplier;
        result.capital_at_risk = result.capital_at_risk * multiplier;
        result.method = SizingMethod::ConfidenceAdjusted;
        result.reasoning.push_str(&format!(" Profile scaling: {}x", multiplier));

        result
    }

    fn apply_drawdown_scaling(&self, mut result: PositionSizeResult, drawdown: &DrawdownState) -> PositionSizeResult {
        match drawdown {
            DrawdownState::Normal => {}
            DrawdownState::Warning { pct } => {
                let scaler = Decimal::ONE - (*pct / Decimal::from(2));
                result.lot_size = result.lot_size * scaler.max(Decimal::from_f64(0.5).unwrap_or(Decimal::ONE));
                result.method = SizingMethod::Conservative;
                result.reasoning.push_str(" Drawdown warning applied.");
            }
            DrawdownState::SoftLimit { pct } => {
                let scaler = Decimal::ONE - (*pct / Decimal::from(3));
                result.lot_size = result.lot_size * scaler.max(Decimal::from_f64(0.25).unwrap_or(Decimal::ONE));
                result.method = SizingMethod::Conservative;
                result.reasoning.push_str(" Drawdown soft limit - reducing size.");
            }
            DrawdownState::HardLimit => {
                result.lot_size = Decimal::ZERO;
                result.capital_at_risk = Decimal::ZERO;
                result.risk_percent = Decimal::ZERO;
                result.method = SizingMethod::Minimal;
                result.reasoning = "Drawdown hard limit - no new positions.".to_string();
            }
            DrawdownState::RecoveryMode => {
                result.lot_size = result.lot_size * Decimal::from_f64(0.5).unwrap_or(Decimal::ONE);
                result.method = SizingMethod::Conservative;
                result.reasoning.push_str(" Recovery mode - half size.");
            }
        }
        result
    }

    fn apply_daily_scaling(&self, mut result: PositionSizeResult, daily: &DailyLimitState) -> PositionSizeResult {
        match daily {
            DailyLimitState::Normal => {}
            DailyLimitState::NearLimit { remaining_pct } => {
                let scaler = *remaining_pct;
                result.lot_size = result.lot_size * scaler;
                result.method = SizingMethod::Conservative;
                result.reasoning.push_str(&format!(" Daily limit near - scaled to {}/100.", remaining_pct * Decimal::from(100)));
            }
            DailyLimitState::LimitReached => {
                result.lot_size = Decimal::ZERO;
                result.capital_at_risk = Decimal::ZERO;
                result.risk_percent = Decimal::ZERO;
                result.method = SizingMethod::Minimal;
                result.reasoning = "Daily loss limit reached.".to_string();
            }
        }
        result
    }

    fn apply_streak_adjustment(&self, mut result: PositionSizeResult, streak: &StreakState) -> PositionSizeResult {
        if streak.consecutive_losses >= 3 {
            let reduction = Decimal::from(streak.consecutive_losses.min(5)) / Decimal::from(10);
            let scaler = Decimal::ONE - reduction;
            result.lot_size = result.lot_size * scaler;
            result.reasoning.push_str(&format!(" Losing streak reduction: {} losses", streak.consecutive_losses));
        }
        result
    }

    fn apply_correlation_reduction(&self, mut result: PositionSizeResult, correlation: &Decimal) -> PositionSizeResult {
        if *correlation > Decimal::from_f64(0.7).unwrap_or(Decimal::ONE) {
            let reduction = (*correlation - Decimal::from_f64(0.7).unwrap_or(Decimal::ONE)) * Decimal::from(2);
            let scaler = (Decimal::ONE - reduction).max(Decimal::from_f64(0.3).unwrap_or(Decimal::ONE));
            result.lot_size = result.lot_size * scaler;
            result.reasoning.push_str(" High correlation reduction.");
        }
        result
    }

    fn apply_exposure_cap(
        &self,
        mut result: PositionSizeResult,
        exposure: &ExposureMetrics,
        equity: Decimal,
    ) -> PositionSizeResult {
        let max_additional_exposure = equity * Decimal::from_f64(0.2).unwrap_or(Decimal::ONE);

        if exposure.total_exposure + result.capital_at_risk > max_additional_exposure {
            let allowed_risk = (max_additional_exposure - exposure.total_exposure).max(Decimal::ZERO);
            if allowed_risk > Decimal::ZERO && result.capital_at_risk > Decimal::ZERO {
                let ratio = allowed_risk / result.capital_at_risk;
                result.lot_size = result.lot_size * ratio;
                result.capital_at_risk = allowed_risk;
                result.reasoning.push_str(" Capped by exposure limit.");
            } else {
                result.lot_size = Decimal::ZERO;
                result.capital_at_risk = Decimal::ZERO;
                result.risk_percent = Decimal::ZERO;
                result.reasoning = "Exposure limit reached.".to_string();
            }
        }

        result
    }
}
