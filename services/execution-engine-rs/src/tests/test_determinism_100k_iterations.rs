use rust_decimal_macros::dec;
use crate::execution::ExecutionRequest;
use crate::order::{OrderId, OrderSide, OrderType, TimeInForce};
use crate::validation::OrderValidator;

#[test]
fn test_determinism_100k_iterations() {
    let order_id = OrderId::new();
    let req = ExecutionRequest {
        order_id,
        symbol: "BTCUSD".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        size: dec!(1.5),
        price: Some(dec!(30000.0)),
        time_in_force: TimeInForce::GoodTillCancel,
    };

    let stop_loss = Some(dec!(29000.0));
    let take_profit = Some(dec!(35000.0));

    let mut success_count = 0;
    for _ in 0..100_000 {
        let res = OrderValidator::validate_request(&req, stop_loss, take_profit);
        if res.is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 100_000);
}
