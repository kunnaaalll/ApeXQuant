use crate::brokers::broker::OrderState;
use crate::connection_supervisor::ConnectionState;
use crate::connection_supervisor::ConnectionSupervisor;
use crate::order_reconciliation::MismatchDetector;
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

    use rust_decimal_macros::dec;
    let local_orders = vec![OrderState {
        ticket: "1".to_string(),
        symbol: "BTCUSD".to_string(),
        side: "buy".to_string(),
        order_type: "limit".to_string(),
        volume: dec!(1.0),
        price: dec!(50000.0),
        status: "OPEN".to_string(),
        timestamp: 0,
    }];
    let broker_orders = vec![OrderState {
        ticket: "1".to_string(),
        symbol: "BTCUSD".to_string(),
        side: "buy".to_string(),
        order_type: "limit".to_string(),
        volume: dec!(1.0),
        price: dec!(50000.0),
        status: "OPEN".to_string(),
        timestamp: 0,
    }];

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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/connect"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/ping"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/positions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(Vec::<crate::brokers::mt5::positions::Mt5Position>::new()),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/orders"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(Vec::<crate::brokers::mt5::orders::Mt5Order>::new()),
        )
        .mount(&mock_server)
        .await;

    let broker =
        crate::brokers::mt5::adapter::Mt5Adapter::new("MT5".to_string(), mock_server.uri());
    broker.connect().await.unwrap();

    let local_orders = vec![];
    let local_positions = vec![];

    let mut successful_recoveries = 0;

    for _ in 0..100_000 {
        let result = engine
            .recover(&broker, &local_orders, &local_positions)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_parity_achieved);
        successful_recoveries += 1;
    }

    assert_eq!(successful_recoveries, 100_000);
}
