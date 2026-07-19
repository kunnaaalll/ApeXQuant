use crate::circuit_breaker::*;
use rust_decimal::Decimal;

#[test]
fn test_forbidden_transitions() {
    let state = CircuitBreakerState::Frozen;
    assert!(state.transition_to(CircuitBreakerState::Normal).is_err());
    assert!(state.transition_to(CircuitBreakerState::Warning).is_err());

    let state = CircuitBreakerState::Critical;
    assert!(state.transition_to(CircuitBreakerState::Normal).is_err());
}

#[test]
fn test_sequential_recovery() {
    let mut state = CircuitBreakerState::Frozen;

    // Frozen -> Critical -> Restricted -> Warning -> Normal
    state = state.transition_to(CircuitBreakerState::Critical).unwrap();
    assert_eq!(state, CircuitBreakerState::Critical);

    state = state
        .transition_to(CircuitBreakerState::Restricted)
        .unwrap();
    assert_eq!(state, CircuitBreakerState::Restricted);

    state = state.transition_to(CircuitBreakerState::Warning).unwrap();
    assert_eq!(state, CircuitBreakerState::Warning);

    state = state.transition_to(CircuitBreakerState::Normal).unwrap();
    assert_eq!(state, CircuitBreakerState::Normal);
}

#[test]
fn test_drawdown_capacity_non_negative() {
    let assessment = DrawdownLimitAssessment::new(Decimal::new(100, 0), Decimal::new(50, 0));
    assert_eq!(assessment.remaining_drawdown_capacity, Decimal::ZERO);
    assert_eq!(assessment.state, DrawdownState::Frozen);
}

#[test]
fn test_leverage_bounds_non_negative() {
    let assessment = LeverageAssessment::new(
        Decimal::new(-10, 0),
        Decimal::new(-5, 0),
        Decimal::new(-2, 0),
    );
    assert_eq!(assessment.gross_leverage, Decimal::ZERO);
    assert_eq!(assessment.effective_leverage, Decimal::ZERO);
    assert_eq!(assessment.hidden_leverage, Decimal::ZERO);
}

#[test]
fn test_severity_clamp() {
    let assessment = RiskSeverityAssessment::new(Decimal::new(150, 0));
    assert_eq!(
        assessment.overall_risk_restriction_score,
        Decimal::new(100, 0)
    );
    assert_eq!(assessment.state, SeverityState::Frozen);

    let assessment = RiskSeverityAssessment::new(Decimal::new(-10, 0));
    assert_eq!(assessment.overall_risk_restriction_score, Decimal::ZERO);
    assert_eq!(assessment.state, SeverityState::Normal);
}

#[test]
fn test_cooldown_behavior() {
    let mut cooldown = CooldownModel::new(5);
    cooldown.tick(false);
    cooldown.tick(false);
    assert!(!cooldown.is_cooldown_complete());
    cooldown.tick(true); // Loss
    assert_eq!(cooldown.current_cooldown_ticks, 0);
    assert!(!cooldown.is_cooldown_complete());

    for _ in 0..5 {
        cooldown.tick(false);
    }
    assert!(cooldown.is_cooldown_complete());
}

#[test]
fn test_recovery_decay() {
    let mut recovery = RiskRecoveryModel::new(3, Decimal::new(9, 1)); // 0.9 decay
    recovery.tick(false);
    recovery.tick(false);
    recovery.tick(false);
    assert_eq!(recovery.score, Decimal::new(10, 2)); // 0.10

    recovery.tick(true); // Loss
    assert_eq!(recovery.consecutive_positive_periods, 0);
    assert_eq!(recovery.score, Decimal::new(9, 2)); // 0.1 * 0.9 = 0.09
}

#[test]
fn test_determinism_100k_iterations() {
    let mut cooldown = CooldownModel::new(100);
    let mut recovery = RiskRecoveryModel::new(10, Decimal::new(9, 1));

    for i in 0..100_000 {
        // Deterministic sequence, no randomness
        let is_loss = i % 100 == 0;
        cooldown.tick(is_loss);
        recovery.tick(is_loss);
    }

    // State at 100k should be perfectly deterministic
    // Last loss was at 99900. Since then, 99 ticks.
    assert_eq!(cooldown.current_cooldown_ticks, 99);
    assert!(!cooldown.is_cooldown_complete());

    assert_eq!(recovery.consecutive_positive_periods, 99);
    // Score would max out at 1.0 since it reached 10 consecutive ticks many times
    assert_eq!(recovery.score, Decimal::ONE);
    assert_eq!(recovery.state, RecoveryState::Healthy);
}

#[test]
fn test_event_replay() {
    let events = vec![
        CircuitBreakerEvent::StateChanged {
            from: CircuitBreakerState::Normal,
            to: CircuitBreakerState::Warning,
            timestamp_ms: 1000,
            version: 1,
            reason: "Volatility".to_string(),
        },
        CircuitBreakerEvent::SeverityScoreUpdated {
            new_score: Decimal::new(25, 0),
            timestamp_ms: 1001,
            version: 2,
        },
        CircuitBreakerEvent::DrawdownCapacityUpdated {
            remaining_capacity: Decimal::new(5000, 0),
            timestamp_ms: 1002,
            version: 3,
        },
    ];

    let snapshot = CircuitBreakerSnapshot::replay(&events, 0);

    assert_eq!(snapshot.state, CircuitBreakerState::Warning);
    assert_eq!(snapshot.severity_score, Decimal::new(25, 0));
    assert_eq!(snapshot.remaining_drawdown_capacity, Decimal::new(5000, 0));
    assert_eq!(snapshot.timestamp_ms, 1002);
    assert_eq!(snapshot.version, 3);
}
