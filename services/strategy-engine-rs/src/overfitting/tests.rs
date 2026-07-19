use super::overfit_detector::OverfitDetector;
use super::sample_bias::SampleBias;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

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
    let detector = OverfitDetector::new();

    // Case 1: Healthy ratio
    let healthy_ratio = detector.check_overfit(dec!(1.5), dec!(1.2), 1);
    assert_eq!(healthy_ratio, dec!(1.5) / dec!(1.2) + dec!(0.15));

    // Case 2: Extreme overfit (Negative OOS)
    let extreme = detector.check_overfit(dec!(2.0), dec!(-0.5), 1);
    assert_eq!(extreme, dec!(99.0));

    // Case 3: High complexity penalty
    let complex = detector.check_overfit(dec!(2.0), dec!(1.0), 5);
    assert_eq!(complex, dec!(2.0) + (dec!(5) * dec!(0.15)));
}
