//! Circuit breakers for catastrophic condition protection
use rust_decimal::prelude::FromPrimitive;

use crate::RiskInputs;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use time::OffsetDateTime;

mod hard_breaker;
mod recovery;
mod soft_breaker;

pub use hard_breaker::HardBreaker;
pub use recovery::RecoveryManager;
pub use soft_breaker::SoftBreaker;

/// Circuit breaker state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CircuitBreakerState {
    /// Normal operation
    Normal,
    /// Warning issued
    Warning,
    /// Risk reduced
    ReducedRisk,
    /// Recovery mode
    Recovery,
    /// Hard stop - no new positions
    HardStop,
}

impl CircuitBreakerState {
    /// Whether new positions are allowed
    pub fn allows_trading(&self) -> bool {
        !matches!(self, CircuitBreakerState::HardStop)
    }

    /// Whether position size should be reduced
    pub fn should_reduce_size(&self) -> bool {
        matches!(
            self,
            CircuitBreakerState::ReducedRisk | CircuitBreakerState::Recovery
        )
    }

    /// Position size multiplier for this state
    pub fn position_multiplier(&self) -> Decimal {
        match self {
            CircuitBreakerState::Normal => Decimal::ONE,
            CircuitBreakerState::Warning => Decimal::from_f64(0.9).unwrap(),
            CircuitBreakerState::ReducedRisk => Decimal::from_f64(0.5).unwrap(),
            CircuitBreakerState::Recovery => Decimal::from_f64(0.25).unwrap(),
            CircuitBreakerState::HardStop => Decimal::ZERO,
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CircuitBreakerState::Normal => "Normal operation",
            CircuitBreakerState::Warning => "Warning - conditions deteriorating",
            CircuitBreakerState::ReducedRisk => "Risk reduced - protective measures active",
            CircuitBreakerState::Recovery => "Recovery mode - cautious trading",
            CircuitBreakerState::HardStop => "Hard stop - no new positions",
        }
    }

    /// Check if state can transition to target
    pub fn can_transition_to(&self, target: CircuitBreakerState) -> bool {
        match (self, target) {
            (CircuitBreakerState::HardStop, CircuitBreakerState::Normal) => false, // Must go through recovery
            (CircuitBreakerState::Normal, CircuitBreakerState::HardStop) => true,
            _ => true,
        }
    }
}

/// Individual circuit breaker
pub trait CircuitBreaker: Send + Sync {
    /// Name of the breaker
    fn name(&self) -> &str;

    /// Check if breaker should trigger
    fn check(&self, inputs: &RiskInputs) -> Option<CircuitBreakerTrigger>;

    /// Get current state
    fn state(&self) -> CircuitBreakerState;

    /// Reset the breaker
    fn reset(&mut self);
}

/// Trigger information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerTrigger {
    /// Breaker name
    pub breaker: String,
    /// Reason for trigger
    pub reason: String,
    /// Severity level
    pub severity: BreakerSeverity,
    /// Recommended state
    pub recommended_state: CircuitBreakerState,
    /// Timestamp
    pub timestamp: OffsetDateTime,
    /// How long to stay in triggered state
    pub duration: Duration,
}

/// Breaker severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BreakerSeverity {
    /// Informational only
    Info,
    /// Warning - reduce risk
    Warning,
    /// Critical - stop new positions
    Critical,
    /// Catastrophic - emergency shutdown
    Catastrophic,
}

impl BreakerSeverity {
    /// Get corresponding state
    pub fn to_state(&self) -> CircuitBreakerState {
        match self {
            BreakerSeverity::Info => CircuitBreakerState::Normal,
            BreakerSeverity::Warning => CircuitBreakerState::Warning,
            BreakerSeverity::Critical => CircuitBreakerState::HardStop,
            BreakerSeverity::Catastrophic => CircuitBreakerState::HardStop,
        }
    }
}

/// Registry of all circuit breakers
pub struct CircuitBreakerRegistry {
    breakers: Vec<Box<dyn CircuitBreaker>>,
    current_state: CircuitBreakerState,
    last_trigger: Option<CircuitBreakerTrigger>,
    state_entered_at: Instant,
}

impl CircuitBreakerRegistry {
    /// Create new registry with default breakers
    pub fn new() -> Self {
        let mut registry = Self {
            breakers: Vec::new(),
            current_state: CircuitBreakerState::Normal,
            last_trigger: None,
            state_entered_at: Instant::now(),
        };

        registry.register_default_breakers();
        registry
    }

    /// Register a circuit breaker
    pub fn register(&mut self, breaker: Box<dyn CircuitBreaker>) {
        self.breakers.push(breaker);
    }

    /// Check all breakers and return most severe trigger
    pub fn check(&self, inputs: &RiskInputs) -> Option<&dyn CircuitBreaker> {
        for breaker in &self.breakers {
            if let Some(_trigger) = breaker.check(inputs) {
                return Some(breaker.as_ref());
            }
        }
        None
    }

    /// Get current overall state
    pub fn state(&self) -> CircuitBreakerState {
        self.current_state
    }

    /// Get last trigger info
    pub fn last_trigger(&self) -> Option<&CircuitBreakerTrigger> {
        self.last_trigger.as_ref()
    }

    /// Update state based on breaker checks
    pub fn update(&mut self, inputs: &RiskInputs) {
        let mut max_severity = BreakerSeverity::Info;
        let mut latest_trigger: Option<CircuitBreakerTrigger> = None;

        for breaker in &self.breakers {
            if let Some(trigger) = breaker.check(inputs) {
                if Self::severity_rank(&trigger.severity) > Self::severity_rank(&max_severity) {
                    max_severity = trigger.severity.clone();
                    latest_trigger = Some(trigger);
                }
            }
        }

        let new_state = max_severity.to_state();

        if new_state != self.current_state {
            self.transition_to(new_state, latest_trigger);
        }
    }

    /// Check if can exit current state
    pub fn can_exit_state(&self) -> bool {
        let elapsed = self.state_entered_at.elapsed();

        match self.current_state {
            CircuitBreakerState::HardStop => elapsed > Duration::from_secs(3600), // 1 hour cooldown
            CircuitBreakerState::ReducedRisk => elapsed > Duration::from_secs(1800), // 30 min
            CircuitBreakerState::Recovery => elapsed > Duration::from_secs(900), // 15 min
            CircuitBreakerState::Warning => elapsed > Duration::from_secs(300), // 5 min
            CircuitBreakerState::Normal => true,
        }
    }

    /// Manual reset (use with caution)
    pub fn manual_reset(&mut self) {
        self.current_state = CircuitBreakerState::Normal;
        self.last_trigger = None;
        self.state_entered_at = Instant::now();

        for breaker in &mut self.breakers {
            breaker.reset();
        }
    }

    fn register_default_breakers(&mut self) {
        // Drawdown breaker
        self.register(Box::new(HardBreaker::drawdown(
            Decimal::from_f64(0.15).unwrap(), // 15% hard limit
            Decimal::from_f64(0.10).unwrap(), // 10% soft limit
        )));

        // Consecutive loss breaker
        self.register(Box::new(SoftBreaker::consecutive_losses(5)));

        // Daily loss breaker
        self.register(Box::new(HardBreaker::daily_loss(
            Decimal::from_f64(0.10).unwrap(), // 10% daily limit
        )));

        // Exposure concentration breaker
        self.register(Box::new(SoftBreaker::exposure_concentration(
            Decimal::from_f64(0.7).unwrap(), // 70% correlation threshold
        )));
    }

    fn transition_to(&mut self, new_state: CircuitBreakerState, trigger: Option<CircuitBreakerTrigger>) {
        if self.current_state.can_transition_to(new_state) {
            self.current_state = new_state;
            self.last_trigger = trigger;
            self.state_entered_at = Instant::now();
        }
    }

    fn severity_rank(severity: &BreakerSeverity) -> u8 {
        match severity {
            BreakerSeverity::Info => 0,
            BreakerSeverity::Warning => 1,
            BreakerSeverity::Critical => 2,
            BreakerSeverity::Catastrophic => 3,
        }
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_allows_trading() {
        assert!(CircuitBreakerState::Normal.allows_trading());
        assert!(CircuitBreakerState::Warning.allows_trading());
        assert!(CircuitBreakerState::ReducedRisk.allows_trading());
        assert!(!CircuitBreakerState::HardStop.allows_trading());
    }

    #[test]
    fn test_state_multipliers() {
        assert_eq!(CircuitBreakerState::Normal.position_multiplier(), Decimal::ONE);
        assert!(CircuitBreakerState::ReducedRisk.position_multiplier() < Decimal::ONE);
        assert_eq!(CircuitBreakerState::HardStop.position_multiplier(), Decimal::ZERO);
    }

    #[test]
    fn test_state_transitions() {
        assert!(
            CircuitBreakerState::Normal.can_transition_to(CircuitBreakerState::HardStop)
        );
        // HardStop cannot directly go to Normal
        assert!(
            !CircuitBreakerState::HardStop.can_transition_to(CircuitBreakerState::Normal)
        );
    }

    #[test]
    fn test_severity_to_state() {
        assert_eq!(
            BreakerSeverity::Warning.to_state(),
            CircuitBreakerState::Warning
        );
        assert_eq!(
            BreakerSeverity::Critical.to_state(),
            CircuitBreakerState::HardStop
        );
    }
}
