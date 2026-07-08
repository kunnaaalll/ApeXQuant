use rust_decimal_macros::dec;

use super::anomaly_detection::{AnomalyDetectionEngine, AnomalyType};
use super::circuit_breaker::{ExecutionError, ExecutionProtectionState};
use super::cooldown::CooldownEngine;
use super::events::ExecutionRiskEvent;
use super::failure_tracker::FailureTracker;
use super::fill_quality_guards::FillQualityGuards;
use super::latency_guards::LatencyGuards;
use super::liquidity_guards::LiquidityGuards;
use super::recovery::{RecoveryEngine, RecoveryState};
use super::rejection_tracker::RejectionTracker;
use super::severity::Severity;
use super::snapshots::ExecutionRiskSnapshot;
use super::spread_guards::SpreadGuards;

#[test]
fn test_state_transition_rules() {
    let mut state = ExecutionProtectionState::Frozen;

    // Direct jump to normal should fail
    assert_eq!(
        state.transition(ExecutionProtectionState::Normal),
        Err(ExecutionError::IllegalTransition)
    );

    // Sequential recovery should succeed
    assert_eq!(state.transition(ExecutionProtectionState::Critical), Ok(()));
    assert_eq!(
        state.transition(ExecutionProtectionState::Restricted),
        Ok(())
    );
    assert_eq!(state.transition(ExecutionProtectionState::Warning), Ok(()));
    assert_eq!(state.transition(ExecutionProtectionState::Normal), Ok(()));

    // Escalation allows skipping
    assert_eq!(state.transition(ExecutionProtectionState::Frozen), Ok(()));
}

#[test]
fn test_spread_bounds() {
    // Current < Average -> 0
    let guards = SpreadGuards::new(dec!(0.5), dec!(1.0));
    assert_eq!(guards.get_score(), 0);

    // Current = 3x Average -> middle score
    let guards = SpreadGuards::new(dec!(3.0), dec!(1.0));
    let score = guards.get_score();
    assert!(score > 0 && score < 100);

    // Current = 10x Average -> clamped to 100
    let guards = SpreadGuards::new(dec!(10.0), dec!(1.0));
    assert_eq!(guards.get_score(), 100);
}

#[test]
fn test_latency_bounds() {
    let guards = LatencyGuards::new(5, 5, 0); // 10ms
    assert_eq!(guards.get_score(), 0);

    let guards = LatencyGuards::new(100, 100, 50); // 250ms
    assert_eq!(guards.get_score(), 100);

    // Test saturating add to avoid overflow
    let guards = LatencyGuards::new(u32::MAX, u32::MAX, 10);
    assert_eq!(guards.total_latency_ms(), u32::MAX);
    assert_eq!(guards.get_score(), 100);
}

#[test]
fn test_rejection_lock() {
    let mut tracker = RejectionTracker::new(0, dec!(0.0));
    assert_eq!(
        tracker.get_protection_state(),
        ExecutionProtectionState::Normal
    );

    tracker.consecutive_rejections = 5;
    assert_eq!(
        tracker.get_protection_state(),
        ExecutionProtectionState::Frozen
    );

    tracker.consecutive_rejections = 0;
    tracker.rolling_rejection_rate = dec!(0.25);
    assert_eq!(
        tracker.get_protection_state(),
        ExecutionProtectionState::Frozen
    );
}

#[test]
fn test_failure_recovery() {
    let cooldown = CooldownEngine::new(3, 5);
    let mut recovery = RecoveryEngine::new(cooldown);
    let mut state = ExecutionProtectionState::Frozen;

    recovery.record_failure();
    assert_eq!(recovery.current_state, RecoveryState::Locked);

    // Try recovery while locked -> does nothing
    recovery.attempt_recovery(&mut state).unwrap();
    assert_eq!(state, ExecutionProtectionState::Frozen);

    // Fulfill cooldown
    recovery.cooldown.stable_cycles_completed = 3;
    recovery.cooldown.fills_completed = 5;

    // Should transition to recovering AND advance state to Critical in the same tick
    recovery.attempt_recovery(&mut state).unwrap();
    assert_eq!(recovery.current_state, RecoveryState::Recovering);
    assert_eq!(state, ExecutionProtectionState::Critical);

    // Run recovery sequentially
    recovery.attempt_recovery(&mut state).unwrap();
    assert_eq!(state, ExecutionProtectionState::Restricted);
}

#[test]
fn test_cooldown_reset() {
    let cooldown = CooldownEngine::new(3, 5);
    let mut recovery = RecoveryEngine::new(cooldown);

    recovery.cooldown.stable_cycles_completed = 2;
    recovery.record_failure(); // Should reset
    assert_eq!(recovery.cooldown.stable_cycles_completed, 0);
}

#[test]
fn test_anomaly_detection() {
    let spread = SpreadGuards::new(dec!(6.0), dec!(1.0)); // > 5x => Catastrophic
    let report = AnomalyDetectionEngine::detect_spread_anomalies(&spread).unwrap();
    assert_eq!(report.severity, Severity::Catastrophic);
    assert_eq!(report.anomaly_type, AnomalyType::SpreadExplosion);
}

#[test]
fn test_event_rebuild() {
    let events = vec![
        ExecutionRiskEvent::StateTransition {
            from: ExecutionProtectionState::Normal,
            to: ExecutionProtectionState::Critical,
            reason: "test".to_string(),
        },
        ExecutionRiskEvent::FailureRecorded {
            error_type: "timeout".to_string(),
            failure_score: 50,
        },
    ];

    let snapshot = ExecutionRiskSnapshot::rebuild_from_events(&events);
    assert_eq!(snapshot.current_state, ExecutionProtectionState::Critical);
    assert_eq!(snapshot.failure_score, 50);
}

#[test]
fn test_determinism_100k_iterations() {
    // 100,000 loops, verify identical outputs, zero drift, zero panics, zero randomness
    let mut _state = ExecutionProtectionState::Normal;

    for _i in 0..100_000 {
        let spread = SpreadGuards::new(dec!(1.1), dec!(1.0));
        let slippage = super::slippage_guards::SlippageGuards::new(dec!(0.0), dec!(0.0));
        let liquidity = LiquidityGuards::new(dec!(100.0), dec!(1.0), dec!(0.0), dec!(50.0));
        let latency = LatencyGuards::new(5, 5, 5);
        let fill_quality = FillQualityGuards::new(0, 0, 100);
        let rejections = RejectionTracker::new(0, dec!(0.0));
        let failures = FailureTracker::new(0, 0, 0, 0);

        let score = super::escalation::EscalationEngine::compute_execution_risk_score(
            &spread,
            &slippage,
            &liquidity,
            &latency,
            &fill_quality,
            &rejections,
            &failures,
        );

        let next_state = super::escalation::EscalationEngine::determine_protection_state(score);

        // State should remain normal throughout all 100k iterations
        assert_eq!(next_state, ExecutionProtectionState::Normal);
        _state = next_state;

        // No panic, no unsafe code
    }
}
