use crate::fills::Fill;
use crate::order::OrderId;
use rust_decimal_macros::dec;

#[test]
fn test_fill_values_non_negative() {
    let fill = Fill::new(
        OrderId::new(),
        dec!(1.1000),
        dec!(1.0),
        dec!(0.0001),
        dec!(0.00005),
    );

    assert!(fill.price >= dec!(0));
    assert!(fill.quantity >= dec!(0));
    assert!(fill.commission >= dec!(0));
    assert!(fill.slippage >= dec!(0));
}
