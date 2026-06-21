use rust_decimal_macros::dec;
use crate::execution::ExecutionRequest;
use crate::order::{OrderId, OrderSide, OrderType, TimeInForce};
use crate::validation::{OrderValidator, ValidationError};

#[test]
fn test_validation_rejects_invalid_orders() {
    let req = ExecutionRequest {
        order_id: OrderId::new(),
        symbol: "ETHUSD".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        size: dec!(-1.0),
        price: Some(dec!(2000.0)),
        time_in_force: TimeInForce::GoodTillCancel,
    };

    // Invalid Size
    let res = OrderValidator::validate_request(&req, None, None);
    assert_eq!(res, Err(ValidationError::InvalidSize));

    let mut req2 = req.clone();
    req2.size = dec!(1.0);
    req2.price = Some(dec!(-10.0));

    // Negative Price
    let res2 = OrderValidator::validate_request(&req2, None, None);
    assert_eq!(res2, Err(ValidationError::NegativePrice));

    let mut req3 = req.clone();
    req3.size = dec!(1.0);
    req3.price = Some(dec!(2000.0));

    // Invalid Stop Loss for Buy (SL > Price)
    let res3 = OrderValidator::validate_request(&req3, Some(dec!(2100.0)), None);
    assert_eq!(res3, Err(ValidationError::InvalidStopLoss));
    
    // Invalid Take Profit for Buy (TP < Price)
    let res4 = OrderValidator::validate_request(&req3, None, Some(dec!(1900.0)));
    assert_eq!(res4, Err(ValidationError::InvalidTakeProfit));
    
    let mut req_sell = req.clone();
    req_sell.size = dec!(1.0);
    req_sell.side = OrderSide::Sell;
    
    // Invalid Stop Loss for Sell (SL < Price)
    let res5 = OrderValidator::validate_request(&req_sell, Some(dec!(1900.0)), None);
    assert_eq!(res5, Err(ValidationError::InvalidStopLoss));
    
    // Invalid Take Profit for Sell (TP > Price)
    let res6 = OrderValidator::validate_request(&req_sell, None, Some(dec!(2100.0)));
    assert_eq!(res6, Err(ValidationError::InvalidTakeProfit));
    
    // Limit order without price
    let mut req_limit_no_price = req.clone();
    req_limit_no_price.size = dec!(1.0);
    req_limit_no_price.price = None;
    let res7 = OrderValidator::validate_request(&req_limit_no_price, None, None);
    assert_eq!(res7, Err(ValidationError::InvalidPrice));
}
