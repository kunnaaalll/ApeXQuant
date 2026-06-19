use super::sample_bias::SampleBias;
use rust_decimal::Decimal;

#[test]
fn test_sample_penalties() {
    let bias_under_20 = SampleBias::calculate(15);
    assert_eq!(bias_under_20.multiplier, Decimal::new(20, 2));

    let bias_under_50 = SampleBias::calculate(49);
    assert_eq!(bias_under_50.multiplier, Decimal::new(40, 2));

    let bias_over_300 = SampleBias::calculate(350);
    assert_eq!(bias_over_300.multiplier, Decimal::new(100, 2));
}

#[test]
fn test_oos_ratio() {
    // Placeholder test for OOS ratio logic to be implemented
    assert_eq!(1, 1);
}
