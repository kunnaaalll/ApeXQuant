//! Recovery mode management
use rust_decimal::prelude::FromPrimitive;

use super::{BreakerSeverity, CircuitBreakerState};
use rust_decimal::Decimal;
use std::time::{Duration, Instant};

/// Manages recovery from circuit breaker states
pub struct RecoveryManager {
    /// Current recovery state
    state: RecoveryState,
    /// When recovery started
    started_at: Option<Instant>,
    /// Recovery target equity level
    target_equity: Decimal,
    /// Original equity (for recovery calculation)
    original_equity: Decimal,
    /// Step-by-step recovery plan
    plan: RecoveryPlan,
}

/// Recovery states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecoveryState {
    /// No recovery needed
    None,
    /// In soft recovery
    Soft,
    /// In medium recovery
    Medium,
    /// In hard recovery
    Hard,
    /// Recovery complete
    Complete,
}

/// Detailed recovery plan
#[derive(Debug, Clone)]
pub struct RecoveryPlan {
    /// Current phase
    pub phase: u8,
    /// Total phases
    pub total_phases: u8,
    /// Position size multiplier for this phase
    pub position_multiplier: Decimal,
    /// Risk percent multiplier
    pub risk_multiplier: Decimal,
    /// Duration for this phase
    pub phase_duration: Duration,
    /// Success criteria for advancing
    pub success_criteria: RecoveryCriteria,
}

/// Criteria for completing recovery phase
#[derive(Debug, Clone)]
pub struct RecoveryCriteria {
    /// Minimum profit to advance (% of equity)
    pub min_profit_pct: Decimal,
    /// Minimum trades required
    pub min_trades: u32,
    /// Minimum win rate
    pub min_win_rate: Decimal,
    /// Time in phase
    pub time_in_phase: Duration,
}

impl RecoveryManager {
    /// Create new recovery manager
    pub fn new() -> Self {
        Self {
            state: RecoveryState::None,
            started_at: None,
            target_equity: Decimal::ZERO,
            original_equity: Decimal::ZERO,
            plan: RecoveryPlan::default_soft(),
        }
    }

    /// Start recovery from a breaker state
    pub fn start_recovery(
        &mut self,
        breaker_state: &CircuitBreakerState,
        current_equity: Decimal,
    ) -> RecoveryPlan {
        self.state = self.state_from_breaker(breaker_state);
        self.original_equity = current_equity;
        self.started_at = Some(Instant::now());

        // Set recovery target: recover to 50% of drawdown
        self.target_equity = current_equity * Decimal::from_f64(1.05).unwrap();

        self.plan = match self.state {
            RecoveryState::Soft => RecoveryPlan::default_soft(),
            RecoveryState::Medium => RecoveryPlan::default_medium(),
            RecoveryState::Hard => RecoveryPlan::default_hard(),
            _ => RecoveryPlan::default_soft(),
        };

        self.plan.clone()
    }

    /// Update recovery progress
    pub fn update(&mut self, current_equity: Decimal, trades: u32, wins: u32) {
        if !self.is_in_recovery() {
            return;
        }

        // Check if criteria met for next phase
        if self.meets_criteria(current_equity, trades, wins) {
            self.advance_phase();
        }

        // Check if recovery complete
        if current_equity >= self.target_equity {
            self.state = RecoveryState::Complete;
        }
    }

    /// Get current position size multiplier
    pub fn position_multiplier(&self) -> Decimal {
        self.plan.position_multiplier
    }

    /// Get current risk multiplier
    pub fn risk_multiplier(&self) -> Decimal {
        self.plan.risk_multiplier
    }

    /// Check if currently in recovery
    pub fn is_in_recovery(&self) -> bool {
        !matches!(
            self.state,
            RecoveryState::None | RecoveryState::Complete
        )
    }

    /// Get human-readable recovery status
    pub fn status(&self) -> String {
        match self.state {
            RecoveryState::None => "No recovery in progress".to_string(),
            RecoveryState::Complete => "Recovery complete".to_string(),
            _ => format!(
                "Recovery phase {}/{}: {}% position size, {}% risk",
                self.plan.phase,
                self.plan.total_phases,
                self.plan.position_multiplier * Decimal::from(100),
                self.plan.risk_multiplier * Decimal::from(100)
            ),
        }
    }

    /// Check if can transition from current breaker state
    pub fn can_exit(&self, breaker_state: CircuitBreakerState) -> bool {
        match breaker_state {
            CircuitBreakerState::HardStop => {
                // Must be in recovery and meet time minimum
                self.is_in_recovery()
                    && self.started_at.map_or(false, |s| {
                        s.elapsed() > Duration::from_secs(3600) // 1 hour min
                    })
            }
            CircuitBreakerState::Recovery => {
                // Can exit if recovery criteria met
                self.state == RecoveryState::Complete
            }
            _ => true,
        }
    }

    fn state_from_breaker(&self, state: &CircuitBreakerState) -> RecoveryState {
        match state {
            CircuitBreakerState::Normal => RecoveryState::None,
            CircuitBreakerState::Warning => RecoveryState::Soft,
            CircuitBreakerState::ReducedRisk => RecoveryState::Medium,
            CircuitBreakerState::Recovery => RecoveryState::Hard,
            CircuitBreakerState::HardStop => RecoveryState::Hard,
        }
    }

    fn meets_criteria(&self, equity: Decimal, trades: u32, wins: u32) -> bool {
        let criteria = &self.plan.success_criteria;

        // Check profit
        let profit_pct = (equity - self.original_equity) / self.original_equity;
        if profit_pct < criteria.min_profit_pct {
            return false;
        }

        // Check trade count
        if trades < criteria.min_trades {
            return false;
        }

        // Check win rate
        if trades > 0 {
            let win_rate = Decimal::from(wins) / Decimal::from(trades);
            if win_rate < criteria.min_win_rate {
                return false;
            }
        }

        // Check time
        if let Some(started) = self.started_at {
            if started.elapsed() < criteria.time_in_phase {
                return false;
            }
        }

        true
    }

    fn advance_phase(&mut self) {
        if self.plan.phase < self.plan.total_phases {
            self.plan.phase += 1;

            // Gradually restore normal sizing
            let progress = Decimal::from(self.plan.phase) / Decimal::from(self.plan.total_phases);
            self.plan.position_multiplier = self
                .plan
                .position_multiplier
                .max(progress * Decimal::from_f64(0.8).unwrap());
            self.plan.risk_multiplier = self
                .plan
                .risk_multiplier
                .max(progress * Decimal::from_f64(0.8).unwrap());
        } else {
            self.state = RecoveryState::Complete;
        }
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryPlan {
    /// Default soft recovery plan
    pub fn default_soft() -> Self {
        Self {
            phase: 1,
            total_phases: 3,
            position_multiplier: Decimal::from_f64(0.75).unwrap(),
            risk_multiplier: Decimal::from_f64(0.75).unwrap(),
            phase_duration: Duration::from_secs(1800), // 30 min
            success_criteria: RecoveryCriteria::default_soft(),
        }
    }

    /// Default medium recovery plan
    pub fn default_medium() -> Self {
        Self {
            phase: 1,
            total_phases: 5,
            position_multiplier: Decimal::from_f64(0.5).unwrap(),
            risk_multiplier: Decimal::from_f64(0.5).unwrap(),
            phase_duration: Duration::from_secs(3600), // 1 hour
            success_criteria: RecoveryCriteria::default_medium(),
        }
    }

    /// Default hard recovery plan
    pub fn default_hard() -> Self {
        Self {
            phase: 1,
            total_phases: 10,
            position_multiplier: Decimal::from_f64(0.25).unwrap(),
            risk_multiplier: Decimal::from_f64(0.25).unwrap(),
            phase_duration: Duration::from_secs(7200), // 2 hours
            success_criteria: RecoveryCriteria::default_hard(),
        }
    }
}

impl RecoveryCriteria {
    /// Soft recovery criteria
    pub fn default_soft() -> Self {
        Self {
            min_profit_pct: Decimal::from_f64(0.005).unwrap(), // 0.5%
            min_trades: 3,
            min_win_rate: Decimal::from_f64(0.4).unwrap(),
            time_in_phase: Duration::from_secs(1800),
        }
    }

    /// Medium recovery criteria
    pub fn default_medium() -> Self {
        Self {
            min_profit_pct: Decimal::from_f64(0.01).unwrap(), // 1%
            min_trades: 5,
            min_win_rate: Decimal::from_f64(0.45).unwrap(),
            time_in_phase: Duration::from_secs(3600),
        }
    }

    /// Hard recovery criteria
    pub fn default_hard() -> Self {
        Self {
            min_profit_pct: Decimal::from_f64(0.02).unwrap(), // 2%
            min_trades: 10,
            min_win_rate: Decimal::from_f64(0.5).unwrap(),
            time_in_phase: Duration::from_secs(7200),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_state_transitions() {
        let mut manager = RecoveryManager::new();

        assert!(!manager.is_in_recovery());

        let plan = manager.start_recovery(&CircuitBreakerState::HardStop, Decimal::from(9000));

        assert!(manager.is_in_recovery());
        assert_eq!(plan.phase, 1);
        assert!(plan.position_multiplier < Decimal::ONE);
    }

    #[test]
    fn test_recovery_complete() {
        let mut manager = RecoveryManager::new();
        manager.start_recovery(&CircuitBreakerState::Warning, Decimal::from(9000));

        // Target is ~9450 (5% recovery)
        manager.update(Decimal::from(9500), 5, 3);

        assert!(!manager.is_in_recovery() || manager.state == RecoveryState::Complete);
    }

    #[test]
    fn test_position_multipliers() {
        let soft = RecoveryPlan::default_soft();
        let hard = RecoveryPlan::default_hard();

        assert!(soft.position_multiplier > hard.position_multiplier);
    }

    #[test]
    fn test_can_exit_hard_stop() {
        let manager = RecoveryManager::new();

        // Without starting recovery, cannot exit HardStop
        assert!(!manager.can_exit(CircuitBreakerState::HardStop));
    }

    #[test]
    fn test_status_messages() {
        let mut manager = RecoveryManager::new();
        assert!(manager.status().contains("No recovery"));

        manager.start_recovery(&CircuitBreakerState::ReducedRisk, Decimal::from(10000));
        assert!(manager.status().contains("phase"));
    }
}
