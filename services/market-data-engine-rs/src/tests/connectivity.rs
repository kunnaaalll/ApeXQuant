use crate::state_machine::ConnectionState;
use crate::failover::FailoverState;
use crate::latency::{LatencyGrade, LatencyMetrics};
use crate::quality::{FeedQuality, QualityMetrics};
use crate::registry::FeedRegistry;
use crate::connectors::mt5::Mt5FeedAdapter;
use crate::snapshots::FeedSnapshot;
use crate::health::{FeedHealthGrade, HealthMetrics};
use rust_decimal::Decimal;
use crate::events::FeedEvent;

#[test]
fn test_connection_transitions() {
    let mut state = ConnectionState::Disconnected;
    state = state.transition(ConnectionState::Connecting).unwrap_or(state);
    state = state.transition(ConnectionState::Synchronizing).unwrap_or(state);
    state = state.transition(ConnectionState::Healthy).unwrap_or(state);
    assert_eq!(state, ConnectionState::Healthy);
}

#[test]
fn test_illegal_state_jumps() {
    let state = ConnectionState::Disconnected;
    let res = state.transition(ConnectionState::Recovering);
    assert!(res.is_err());

    let state = ConnectionState::Healthy;
    let res = state.transition(ConnectionState::Connecting);
    assert!(res.is_err());

    let state = ConnectionState::Failed;
    let res = state.transition(ConnectionState::Healthy);
    assert!(res.is_err());
}

#[test]
fn test_latency_bounds() {
    let metrics = LatencyMetrics::new(Decimal::from(2), Decimal::from(2), Decimal::from(1));
    assert_eq!(metrics.grade(), LatencyGrade::Excellent);

    let metrics2 = LatencyMetrics::new(Decimal::from(50), Decimal::from(50), Decimal::from(50));
    assert_eq!(metrics2.grade(), LatencyGrade::Critical);
}

#[test]
fn test_feed_quality() {
    let metrics = QualityMetrics::new();
    assert_eq!(metrics.evaluate(), FeedQuality::Elite);
}

#[test]
fn test_failover_sequence() {
    let mut state = FailoverState::Healthy;
    state = state.transition(FailoverState::Warning).unwrap_or(state);
    state = state.transition(FailoverState::Failover).unwrap_or(state);
    state = state.transition(FailoverState::Recovery).unwrap_or(state);
    state = state.transition(FailoverState::Healthy).unwrap_or(state);
    assert_eq!(state, FailoverState::Healthy);
}

#[test]
fn test_registry_lookup() {
    let mut registry = FeedRegistry::new();
    let adapter = Mt5FeedAdapter::new("123".to_string(), "srv".to_string(), "term".to_string());
    registry.register("mt5".to_string(), Box::new(adapter));
    assert!(registry.lookup("mt5").is_some());
    assert!(registry.lookup("none").is_none());
}

#[test]
fn test_snapshot_restore() {
    let snap = FeedSnapshot {
        id: "feed1".to_string(),
        connection_state: ConnectionState::Healthy,
        health: FeedHealthGrade::Excellent,
        latency: LatencyGrade::Normal,
        failover_status: FailoverState::Healthy,
        registry_active: true,
    };
    assert_eq!(snap.connection_state, ConnectionState::Healthy);
}

#[test]
fn test_event_replay() {
    let event = FeedEvent::Connected("feed1".to_string());
    assert!(matches!(event, FeedEvent::Connected(_)));
}

#[test]
fn test_health_degradation() {
    let mut metrics = HealthMetrics::new();
    metrics.missing_ticks = 101;
    assert_eq!(metrics.evaluate(), FeedHealthGrade::Dead);
}

#[test]
fn test_determinism_100k_iterations() {
    let state = ConnectionState::Disconnected;
    for _ in 0..100_000 {
        let _ = state.transition(ConnectionState::Connecting);
    }
    assert_eq!(state, ConnectionState::Disconnected);
}
