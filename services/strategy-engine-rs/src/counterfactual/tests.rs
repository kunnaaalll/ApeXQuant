use super::what_if::CounterfactualResult;
use rust_decimal::Decimal;

#[test]
fn test_difference_sign() {
    let result = CounterfactualResult::new(Decimal::from(100), Decimal::from(150));
    assert_eq!(result.difference, Decimal::from(50));

    let result_neg = CounterfactualResult::new(Decimal::from(100), Decimal::from(50));
    assert_eq!(result_neg.difference, Decimal::from(-50));
}

#[test]
fn test_alternate_history() {
    let result = CounterfactualResult::new(Decimal::from(200), Decimal::from(250));
    assert_eq!(result.actual, Decimal::from(200));
    assert_eq!(result.alternate, Decimal::from(250));
    assert_eq!(result.difference, Decimal::from(50));
}
