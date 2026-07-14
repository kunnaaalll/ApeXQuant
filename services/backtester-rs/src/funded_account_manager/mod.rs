//! Funded Account Manager Module
//!
//! Tracks evaluation challenge progress, funded status, and payout history.
//! All probability and timeline estimates are computed from real performance data.

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FundedManagerError {
    #[error("starting equity must be positive, got {0}")]
    NonPositiveStartingEquity(Decimal),
    #[error("win rate must be in [0, 1], got {0}")]
    InvalidWinRate(Decimal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChallengeStage {
    Phase1,
    Phase2,
    Funded,
}

#[derive(Debug, Clone)]
pub struct FundedAccountState {
    pub account_id: String,
    pub stage: ChallengeStage,
    /// Estimated probability (0.0–1.0) of passing the current challenge phase.
    pub pass_probability: Decimal,
    /// Estimated remaining calendar days to complete the phase at the current pace.
    pub estimated_days_to_complete: u32,
    /// Estimated calendar days until first payout after phase completion.
    pub expected_payout_timeline_days: u32,
    /// Historical payout amounts received.
    pub payout_history: Vec<Decimal>,
}

impl FundedAccountState {
    pub fn new(account_id: String, stage: ChallengeStage) -> Self {
        Self {
            account_id,
            stage,
            pass_probability: Decimal::ZERO,
            estimated_days_to_complete: 0,
            expected_payout_timeline_days: 0,
            payout_history: Vec::new(),
        }
    }
}

/// Phase-specific configuration.
struct PhaseConfig {
    /// Profit target as fraction of starting equity.
    profit_target: Decimal,
    /// Minimum number of trading days required.
    min_trading_days: u32,
    /// Processing days after completion before payout is received.
    payout_processing_days: u32,
}

impl PhaseConfig {
    fn for_phase(stage: &ChallengeStage) -> Self {
        match stage {
            ChallengeStage::Phase1 => PhaseConfig {
                profit_target: Decimal::new(10, 2), // 10%
                min_trading_days: 4,
                payout_processing_days: 7,
            },
            ChallengeStage::Phase2 => PhaseConfig {
                profit_target: Decimal::new(5, 2), // 5%
                min_trading_days: 4,
                payout_processing_days: 5,
            },
            ChallengeStage::Funded => PhaseConfig {
                profit_target: Decimal::ZERO, // No target — ongoing funded account
                min_trading_days: 0,
                payout_processing_days: 3,
            },
        }
    }
}

pub struct FundedManager;

impl FundedManager {
    /// Update state and recalculate pass probability and completion timeline.
    ///
    /// Parameters:
    /// - `state`: mutable account state to update in-place.
    /// - `current_equity`: current equity balance.
    /// - `starting_equity`: equity at phase start.
    /// - `win_rate`: empirical win rate of the strategy in [0, 1].
    /// - `avg_daily_return`: average daily return as a fraction (e.g. 0.003 = 0.3%/day).
    /// - `trading_days_completed`: number of days on which trades were executed.
    pub fn update_progress(
        state: &mut FundedAccountState,
        current_equity: Decimal,
        starting_equity: Decimal,
        win_rate: Decimal,
        avg_daily_return: Decimal,
        trading_days_completed: u32,
    ) -> Result<(), FundedManagerError> {
        if starting_equity <= Decimal::ZERO {
            return Err(FundedManagerError::NonPositiveStartingEquity(
                starting_equity,
            ));
        }
        if win_rate < Decimal::ZERO || win_rate > Decimal::ONE {
            return Err(FundedManagerError::InvalidWinRate(win_rate));
        }

        let config = PhaseConfig::for_phase(&state.stage);

        let current_profit_fraction = (current_equity - starting_equity) / starting_equity;
        let profit_remaining = (config.profit_target - current_profit_fraction).max(Decimal::ZERO);

        // ── Pass Probability ─────────────────────────────────────────────────
        // Logistic function based on current progress toward target + win rate.
        // pass_prob = sigmoid(10 × (progress_fraction × win_rate_bonus - 0.5))
        // where progress_fraction = current_profit / target_profit
        let progress = if config.profit_target > Decimal::ZERO {
            (current_profit_fraction / config.profit_target)
                .min(Decimal::ONE)
                .max(Decimal::ZERO)
        } else {
            Decimal::ONE // Funded accounts have no target — probability = 1
        };

        // Win rate bonus: above 50% win rate provides a positive multiplier.
        let win_rate_bonus = Decimal::ONE + (win_rate - Decimal::new(5, 1)).max(Decimal::ZERO);
        let combined = (progress * win_rate_bonus).min(Decimal::ONE);
        let combined_f = combined.to_f64().unwrap_or(0.5);
        let sigmoid_input = 10.0 * (combined_f - 0.5);
        let pass_prob_f = 1.0 / (1.0 + (-sigmoid_input).exp());
        state.pass_probability =
            Decimal::try_from(pass_prob_f.clamp(0.0, 1.0)).unwrap_or(Decimal::new(5, 1));

        // ── Days to Complete ─────────────────────────────────────────────────
        let avg_daily_f = avg_daily_return.to_f64().unwrap_or(0.003);
        let remaining_f = profit_remaining.to_f64().unwrap_or(0.0);
        let days_for_profit = if avg_daily_f > 0.0 {
            (remaining_f / avg_daily_f).ceil() as u32
        } else {
            999 // No positive return → no convergence estimate
        };
        let days_for_min_trading = config
            .min_trading_days
            .saturating_sub(trading_days_completed);
        state.estimated_days_to_complete = days_for_profit.max(days_for_min_trading);

        // ── Payout Timeline ──────────────────────────────────────────────────
        state.expected_payout_timeline_days =
            state.estimated_days_to_complete + config.payout_processing_days;

        // ── Stage Transition ─────────────────────────────────────────────────
        if current_profit_fraction >= config.profit_target
            && trading_days_completed >= config.min_trading_days
            && config.profit_target > Decimal::ZERO
        {
            state.stage = match state.stage {
                ChallengeStage::Phase1 => ChallengeStage::Phase2,
                ChallengeStage::Phase2 => ChallengeStage::Funded,
                ChallengeStage::Funded => ChallengeStage::Funded, // Already funded
            };
        }

        Ok(())
    }

    /// Record a payout received.
    pub fn record_payout(state: &mut FundedAccountState, amount: Decimal) {
        if amount > Decimal::ZERO {
            state.payout_history.push(amount);
        }
    }

    /// Total payouts received across the account's history.
    pub fn total_payouts(state: &FundedAccountState) -> Decimal {
        state.payout_history.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_progress_low_probability() {
        let mut state = FundedAccountState::new("acc1".to_string(), ChallengeStage::Phase1);
        FundedManager::update_progress(
            &mut state,
            Decimal::from(10_100i64), // 1% profit on $10k — 10% needed
            Decimal::from(10_000i64),
            Decimal::new(55, 2), // 55% win rate
            Decimal::new(3, 3),  // 0.3% avg daily return
            1,
        )
        .expect("ok");
        assert!(
            state.pass_probability < Decimal::new(5, 1),
            "Low progress should yield <50% probability"
        );
        assert!(state.estimated_days_to_complete > 0);
    }

    #[test]
    fn test_target_reached_promotes_stage() {
        let mut state = FundedAccountState::new("acc1".to_string(), ChallengeStage::Phase1);
        FundedManager::update_progress(
            &mut state,
            Decimal::from(11_000i64), // 10% profit
            Decimal::from(10_000i64),
            Decimal::new(65, 2),
            Decimal::new(5, 3),
            4, // meets min trading days
        )
        .expect("ok");
        assert_eq!(state.stage, ChallengeStage::Phase2);
        assert!(state.pass_probability > Decimal::new(7, 1));
    }

    #[test]
    fn test_payout_recording() {
        let mut state = FundedAccountState::new("acc1".to_string(), ChallengeStage::Funded);
        FundedManager::record_payout(&mut state, Decimal::from(500i64));
        FundedManager::record_payout(&mut state, Decimal::from(750i64));
        assert_eq!(FundedManager::total_payouts(&state), Decimal::from(1250i64));
    }

    #[test]
    fn test_invalid_win_rate_returns_error() {
        let mut state = FundedAccountState::new("acc1".to_string(), ChallengeStage::Phase1);
        let result = FundedManager::update_progress(
            &mut state,
            Decimal::from(10_000i64),
            Decimal::from(10_000i64),
            Decimal::from(2i64), // win rate > 1 → invalid
            Decimal::new(3, 3),
            1,
        );
        assert!(matches!(result, Err(FundedManagerError::InvalidWinRate(_))));
    }
}
