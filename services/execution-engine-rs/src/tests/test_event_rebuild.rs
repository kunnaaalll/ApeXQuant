use crate::events::OrderEvent;
use crate::order::{OrderId, OrderSide, OrderType, TimeInForce};
use rust_decimal_macros::dec;

#[test]
fn test_event_rebuild() {
    let order_id = OrderId::new();

    let event = OrderEvent::OrderCreated {
        order_id,
        symbol: "TSLA".to_string(),
        order_type: OrderType::Market,
        side: OrderSide::Sell,
        size: dec!(10),
        price: None,
        time_in_force: TimeInForce::ImmediateOrCancel,
        timestamp: 1672531200,
    };

    let serialized = serde_json::to_string(&event).unwrap();
    let deserialized: OrderEvent = serde_json::from_str(&serialized).unwrap();

    assert_eq!(event, deserialized);
}
