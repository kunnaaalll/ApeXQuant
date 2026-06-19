use super::*;
use rust_decimal_macros::dec;

#[test]
fn test_weight_bounds() {
    let mut optimizer = WeightOptimizer::new(dec!(1.0));
    
    // Test upper bound
    optimizer.update(dec!(3.0));
    assert_eq!(optimizer.weight(), dec!(1.05)); // Only shifted by max 0.05
    assert_eq!(optimizer.state(), WeightState::Increasing);

    // Keep updating until max bound is hit
    for _ in 0..30 {
        optimizer.update(dec!(3.0));
    }
    assert_eq!(optimizer.weight(), dec!(2.00));
    
    // Test lower bound
    let mut optimizer2 = WeightOptimizer::new(dec!(1.0));
    for _ in 0..20 {
        optimizer2.update(dec!(0.1));
    }
    assert_eq!(optimizer2.weight(), dec!(0.50));
}

#[test]
fn test_max_shift_per_cycle() {
    let mut optimizer = WeightOptimizer::new(dec!(1.0));
    
    // Upward shift bounded to 0.05
    optimizer.update(dec!(1.5));
    assert_eq!(optimizer.weight(), dec!(1.05));
    
    // Downward shift bounded to 0.05
    optimizer.update(dec!(0.5));
    assert_eq!(optimizer.weight(), dec!(1.00));
}

#[test]
fn test_decay_model_bounds() {
    let decay_under = DecayModel::new(dec!(0.001));
    let decay_over = DecayModel::new(dec!(0.9));
    let decay_normal = DecayModel::new(dec!(0.2));

    let updated = decay_normal.update(dec!(10.0), dec!(20.0));
    assert_eq!(updated, dec!(12.0)); // 20 * 0.2 + 10 * 0.8 = 4 + 8 = 12

    // Bound checks - we test clamping indirectly via behavior or internal access
    // min alpha = 0.01
    assert_eq!(decay_under.update(dec!(100.0), dec!(200.0)), dec!(101.0)); // 200 * 0.01 + 100 * 0.99 = 2 + 99 = 101
    // max alpha = 0.50
    assert_eq!(decay_over.update(dec!(100.0), dec!(200.0)), dec!(150.0)); // 200 * 0.5 + 100 * 0.5 = 100 + 50 = 150
}
