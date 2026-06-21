use execution_engine::brokers::{
    binance::{account::BinanceAccount, adapter::BinanceAdapter},
    connection::ConnectionState,
    health::BrokerHealth,
    mt5::{account::Mt5Account, adapter::Mt5Adapter},
    registry::{
        failover::FailoverState,
        router::{BrokerRegistry, ExecutionRouter},
        selector::BrokerRole,
    },
    responses::AccountInfo,
    BrokerAdapter,
};
use rust_decimal_macros::dec;
use std::sync::Arc;
use std::time::SystemTime;

#[test]
fn test_connection_transitions() {
    let mut state = ConnectionState::Disconnected;

    // Valid transitions
    assert!(state.transition_to(ConnectionState::Connecting).is_ok());
    assert_eq!(state, ConnectionState::Connecting);

    assert!(state.transition_to(ConnectionState::Connected).is_ok());
    assert_eq!(state, ConnectionState::Connected);

    // Invalid transition
    assert!(state.transition_to(ConnectionState::Connecting).is_err());
    assert_eq!(state, ConnectionState::Connected);

    assert!(state.transition_to(ConnectionState::Degraded).is_ok());
    assert!(state.transition_to(ConnectionState::Reconnecting).is_ok());
    assert!(state.transition_to(ConnectionState::Failed).is_ok());
    assert!(state.transition_to(ConnectionState::Disconnected).is_ok());
}

#[test]
fn test_failover_recovery_sequence() {
    let mut state = FailoverState::Healthy;

    assert!(state.transition_to(FailoverState::Warning).is_ok());
    assert!(state.transition_to(FailoverState::Failover).is_ok());

    // Strict rule: Failover -> Healthy is forbidden
    assert!(state.transition_to(FailoverState::Healthy).is_err());

    // Must go through Recovery
    assert!(state.transition_to(FailoverState::Recovery).is_ok());
    assert!(state.transition_to(FailoverState::Warning).is_ok());
    assert!(state.transition_to(FailoverState::Healthy).is_ok());
}

#[test]
fn test_health_bounds() {
    let health = BrokerHealth::new(dec!(400.0), dec!(99.9), dec!(1000.0), SystemTime::now(), 0);
    assert!(health.is_healthy());
    assert!(!health.is_degraded());

    let degraded = BrokerHealth::new(dec!(600.0), dec!(99.9), dec!(1000.0), SystemTime::now(), 0);
    assert!(!degraded.is_healthy());
    assert!(degraded.is_degraded());

    let downtime = BrokerHealth::new(dec!(100.0), dec!(90.0), dec!(1000.0), SystemTime::now(), 0);
    assert!(!downtime.is_healthy());
    assert!(downtime.is_degraded());
}

#[test]
fn test_mt5_account_state() {
    let account = Mt5Account {
        balance: dec!(10000.0),
        equity: dec!(10500.0),
        free_margin: dec!(9500.0),
        leverage: dec!(100.0),
        margin_level: dec!(1050.0),
    };

    let info: AccountInfo = account.into();
    assert_eq!(info.balance, dec!(10000.0));
    assert_eq!(info.equity, dec!(10500.0));
    assert_eq!(info.margin_level, dec!(1050.0));
}

#[test]
fn test_binance_account_state() {
    let account = BinanceAccount {
        wallet_balance: dec!(10000.0),
        unrealized_pnl: dec!(500.0),
        margin_balance: dec!(10500.0),
        maintenance_margin: dec!(1000.0),
        initial_margin: dec!(1000.0),
        available_balance: dec!(9500.0),
        leverage: dec!(10.0),
    };

    let info: AccountInfo = account.into();
    assert_eq!(info.balance, dec!(10000.0));
    assert_eq!(info.equity, dec!(10500.0));
    assert_eq!(info.margin_level, dec!(1050.0)); // 10500 / 1000 * 100
}

#[test]
fn test_snapshot_rebuild() {
    // Test that serialization and deserialization retains identical state (no precision loss)
    use execution_engine::brokers::snapshots::ConnectionSnapshot;
    let snap = ConnectionSnapshot {
        broker_id: "MT5-1".to_string(),
        state: ConnectionState::Connected,
        timestamp: 1620000000,
    };
    
    let json = serde_json::to_string(&snap).unwrap();
    let rebuilt: ConnectionSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(snap, rebuilt);
}

#[test]
fn test_event_determinism() {
    use execution_engine::brokers::events::{BrokerEvent, HealthEvent};
    let event1 = BrokerEvent::Health(HealthEvent {
        broker_id: "BINANCE-1".to_string(),
        latency_ms: dec!(150.5),
        uptime_percentage: dec!(99.99),
        timestamp: 1620000000,
    });
    
    let event2 = BrokerEvent::Health(HealthEvent {
        broker_id: "BINANCE-1".to_string(),
        latency_ms: dec!(150.5),
        uptime_percentage: dec!(99.99),
        timestamp: 1620000000,
    });
    
    assert_eq!(event1, event2);
}

#[tokio::test]
async fn test_registry_states() {
    let mut registry = BrokerRegistry::new();
    let mt5: Arc<dyn BrokerAdapter> = Arc::new(Mt5Adapter::new("MT5-1".to_string()));
    let binance: Arc<dyn BrokerAdapter> = Arc::new(BinanceAdapter::new("BINANCE-1".to_string()));
    
    registry.register("MT5-1".to_string(), mt5, BrokerRole::Primary);
    registry.register("BINANCE-1".to_string(), binance, BrokerRole::Secondary);

    let primary = registry.get_primary().await.unwrap();
    assert!(primary.health().await.is_err()); // Adapter not fully implemented, returns InternalError
}

#[tokio::test]
async fn test_router_selection() {
    let mut registry = BrokerRegistry::new();
    let mt5: Arc<dyn BrokerAdapter> = Arc::new(Mt5Adapter::new("MT5-1".to_string()));
    registry.register("MT5-1".to_string(), mt5, BrokerRole::Primary);

    let registry_arc = Arc::new(registry);
    let router = ExecutionRouter::new(registry_arc);
    
    let selected = router.route().await;
    assert!(selected.is_ok());
}

#[test]
fn test_determinism_100k_iterations() {
    let mut state = ConnectionState::Disconnected;
    
    for i in 0..100_000 {
        if i % 4 == 0 {
            let _ = state.transition_to(ConnectionState::Connecting);
        } else if i % 4 == 1 {
            let _ = state.transition_to(ConnectionState::Connected);
        } else if i % 4 == 2 {
            let _ = state.transition_to(ConnectionState::Degraded);
        } else {
            let _ = state.transition_to(ConnectionState::Disconnected);
        }
    }
    
    assert_eq!(state, ConnectionState::Disconnected);
}
