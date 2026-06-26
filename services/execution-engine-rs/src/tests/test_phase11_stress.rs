use crate::connection_supervisor::ConnectionSupervisor;
use crate::connection_supervisor::ConnectionState;
use crate::order_reconciliation::{ReconciliationEngine, ReconciliationState, MismatchDetector};
use crate::broker_connectivity::OrderState;
use crate::position_recovery::RecoveryEngine;

#[test]
fn test_100k_reconnect_cycles() {
    let mut supervisor = ConnectionSupervisor::new();
    
    for _ in 0..100_000 {
        supervisor.transition(ConnectionState::Disconnected);
        assert_eq!(supervisor.state, ConnectionState::Disconnected);
        
        supervisor.attempt_reconnect();
        assert_eq!(supervisor.state, ConnectionState::Connecting);
        
        supervisor.transition(ConnectionState::Authenticating);
        
        supervisor.record_auth_success();
        assert_eq!(supervisor.state, ConnectionState::Healthy);
    }
}

#[test]
fn test_1m_reconciliation_comparisons() {
    let detector = MismatchDetector::new();
    
    // We do smaller batches in loop to avoid massive memory allocations
    let local_orders = vec![
        OrderState { id: "1".into(), symbol: "BTCUSD".into(), volume: 1.0, price: 50000.0, is_open: true },
    ];
    let broker_orders = vec![
        OrderState { id: "1".into(), symbol: "BTCUSD".into(), volume: 1.0, price: 50000.0, is_open: true },
    ];

    let mut successful_comparisons = 0;
    
    for _ in 0..1_000_000 {
        let issues = detector.detect(&local_orders, &broker_orders);
        assert!(issues.is_empty());
        successful_comparisons += 1;
    }
    
    assert_eq!(successful_comparisons, 1_000_000);
}

#[tokio::test]
async fn test_100k_recovery_cycles() {
    let engine = RecoveryEngine::new();
    use crate::broker_connectivity::{Mt5Adapter, BrokerAdapter};
    
    let mut broker = Mt5Adapter::new();
    broker.login().await.unwrap();

    let local_orders = vec![];
    let local_positions = vec![];

    let mut successful_recoveries = 0;

    for _ in 0..100_000 {
        let result = engine.recover(&broker, &local_orders, &local_positions).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_parity_achieved);
        successful_recoveries += 1;
    }
    
    assert_eq!(successful_recoveries, 100_000);
}
