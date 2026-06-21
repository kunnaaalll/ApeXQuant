use crate::position::{Position, PositionState};
use crate::order::OrderSide;
use rust_decimal_macros::dec;

#[test]
fn test_position_open_close() {
    let mut position = Position::new(
        "EURUSD".to_string(),
        OrderSide::Buy,
        dec!(1.1000),
        dec!(1.0),
        Some(dec!(1.0900)),
        Some(dec!(1.1100)),
    );

    assert_eq!(position.state, PositionState::Opening);
    
    position.state = PositionState::Open;
    assert_eq!(position.state, PositionState::Open);

    position.state = PositionState::Closing;
    assert_eq!(position.state, PositionState::Closing);

    position.state = PositionState::Closed;
    assert_eq!(position.state, PositionState::Closed);
}
