use super::circuit_breaker::ExecutionProtectionState;
use super::spread_guards::SpreadGuards;
use super::slippage_guards::SlippageGuards;
use super::liquidity_guards::LiquidityGuards;
use super::latency_guards::LatencyGuards;
use super::fill_quality_guards::FillQualityGuards;
use super::rejection_tracker::RejectionTracker;
use super::failure_tracker::FailureTracker;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscalationEngine;

impl EscalationEngine {
    pub fn compute_execution_risk_score(
        spread: &SpreadGuards,
        slippage: &SlippageGuards,
        liquidity: &LiquidityGuards,
        latency: &LatencyGuards,
        fill_quality: &FillQualityGuards,
        rejections: &RejectionTracker,
        failures: &FailureTracker,
    ) -> u32 {
        let mut total_score = 0;

        total_score += spread.get_score().min(100) / 7;
        total_score += slippage.get_penalty_score().min(100) / 7;
        total_score += latency.get_score().min(100) / 7;
        total_score += failures.get_score().min(100) / 7;

        // Map liquidity regime to a 0-100 score equivalent
        let liq_score = match liquidity.get_regime() {
            super::liquidity_guards::LiquidityRegime::Excellent => 0,
            super::liquidity_guards::LiquidityRegime::Normal => 20,
            super::liquidity_guards::LiquidityRegime::Weak => 50,
            super::liquidity_guards::LiquidityRegime::Poor => 80,
            super::liquidity_guards::LiquidityRegime::Broken => 100,
        };
        total_score += liq_score / 7;

        // Map fill grade to a 0-100 score
        let fill_score = match fill_quality.get_grade() {
            super::fill_quality_guards::FillGrade::Elite => 0,
            super::fill_quality_guards::FillGrade::Good => 20,
            super::fill_quality_guards::FillGrade::Normal => 40,
            super::fill_quality_guards::FillGrade::Poor => 80,
            super::fill_quality_guards::FillGrade::Broken => 100,
        };
        total_score += fill_score / 7;

        // Rejection mapping
        let rej_score = match rejections.get_rejection_state() {
            super::rejection_tracker::RejectionState::Normal => 0,
            super::rejection_tracker::RejectionState::Elevated => 30,
            super::rejection_tracker::RejectionState::Danger => 70,
            super::rejection_tracker::RejectionState::Locked => 100,
        };
        total_score += rej_score / 7;

        // Clamp just in case due to division rounding
        total_score.min(100)
    }

    pub fn determine_protection_state(score: u32) -> ExecutionProtectionState {
        let clamped = score.min(100);

        if clamped <= 20 {
            ExecutionProtectionState::Normal
        } else if clamped <= 40 {
            ExecutionProtectionState::Warning
        } else if clamped <= 60 {
            ExecutionProtectionState::Restricted
        } else if clamped <= 80 {
            ExecutionProtectionState::Critical
        } else {
            ExecutionProtectionState::Frozen
        }
    }
}
