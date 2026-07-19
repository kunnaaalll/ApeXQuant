use super::*;
use rust_decimal_macros::dec;

#[test]
fn test_score_bounds() {
    // Everything perfect
    let mut optimizer = SymbolOptimizer::new();
    optimizer.optimize(dec!(1.0), dec!(1.0), dec!(1.0), dec!(0.01), dec!(1.0));
    assert_eq!(optimizer.score(), dec!(100.0));
    assert_eq!(optimizer.grade(), RankingGrade::Elite);

    // Everything terrible
    optimizer.optimize(dec!(-1.0), dec!(0.1), dec!(0.1), dec!(1.0), dec!(0.1));
    assert_eq!(optimizer.score(), dec!(0.0));
    assert_eq!(optimizer.grade(), RankingGrade::Forbidden);

    // Middle ground
    optimizer.optimize(dec!(0.5), dec!(0.5), dec!(0.5), dec!(0.2), dec!(0.5));
    // 0.5*0.5*0.5*0.5 / 0.2 = 0.0625 / 0.2 = 0.3125 * 100 = 31.25 -> Normal
    assert_eq!(optimizer.score(), dec!(31.25));
    assert_eq!(optimizer.grade(), RankingGrade::Weak);
}

#[test]
fn test_zero_division_protection() {
    let mut optimizer = RegimeOptimizer::new();

    // Drawdown is 0.0, should use epsilon to avoid panic
    optimizer.optimize(dec!(1.0), dec!(1.0), dec!(1.0), dec!(0.0), dec!(1.0));
    assert_eq!(optimizer.score(), dec!(100.0)); // Will hit the cap

    // Negative drawdown just in case (should be absolute)
    optimizer.optimize(dec!(0.5), dec!(0.5), dec!(0.5), dec!(-0.2), dec!(0.5));
    assert_eq!(optimizer.score(), dec!(31.25));
}
