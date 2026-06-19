use super::*;
use rust_decimal_macros::dec;

#[test]
fn test_multiplier_bounds() {
    let mut engine = AllocationEngine::new();

    // High risk scenarios should bottom out at 0.25
    engine.compute(dec!(0.0), dec!(0.0), dec!(100.0), dec!(100.0), dec!(0.0));
    assert_eq!(engine.state().multiplier, dec!(0.25));
    assert_eq!(engine.state().exposure, ExposureState::Block);

    // High safety scenarios should max out at 2.00
    engine.compute(dec!(100.0), dec!(100.0), dec!(0.0), dec!(0.0), dec!(100.0));
    assert_eq!(engine.state().multiplier, dec!(2.00));
    assert_eq!(engine.state().exposure, ExposureState::IncreaseExposure);

    // Normal scenarios
    engine.compute(dec!(50.0), dec!(50.0), dec!(0.0), dec!(0.0), dec!(50.0));
    assert_eq!(engine.state().multiplier, dec!(1.10));
    assert_eq!(engine.state().exposure, ExposureState::SlightIncrease);
}
