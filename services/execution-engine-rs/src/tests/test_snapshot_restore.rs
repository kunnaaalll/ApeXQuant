use crate::snapshots::{ExecutionSnapshot, OrderSnapshot};
use crate::order::{OrderId, OrderType, OrderSide, OrderStatus, TimeInForce};
use crate::state::ExecutionState;
use rust_decimal_macros::dec;

#[test]
fn test_snapshot_restore() {
    let snapshot = ExecutionSnapshot {
        version: 1,
        timestamp: 1672531200,
        orders: vec![OrderSnapshot {
            order_id: OrderId::new(),
            symbol: "AAPL".to_string(),
            order_type: OrderType::Limit,
            side: OrderSide::Buy,
            status: OrderStatus::Pending,
            state: ExecutionState::Idle,
            size: dec!(100),
            price: Some(dec!(150.0)),
            filled_quantity: dec!(0),
            time_in_force: TimeInForce::GoodTillCancel,
        }],
        positions: vec![],
    };

    let serialized = serde_json::to_string(&snapshot).unwrap();
    let deserialized: ExecutionSnapshot = serde_json::from_str(&serialized).unwrap();

    assert_eq!(snapshot, deserialized);
}
