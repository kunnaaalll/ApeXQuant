// src/analytics/math.rs
use rust_decimal::Decimal;

/// Safe division that returns a fallback value if the denominator is 0.0.
#[inline]
pub fn safe_divide(numerator: Decimal, denominator: Decimal, fallback: Decimal) -> Decimal {
    if denominator.is_zero() {
        return fallback;
    }
    match numerator.checked_div(denominator) {
        Some(val) => val,
        None => fallback,
    }
}

/// Ensures a value is strictly finite and bounded, reverting to fallback otherwise.
/// Decimal values are inherently finite, so this just returns the value.
#[inline]
pub fn safe_finite(value: Decimal, _fallback: Decimal) -> Decimal {
    value
}

/// Clamps a value between a min and max safely.
#[inline]
pub fn safe_clamp(value: Decimal, min: Decimal, max: Decimal, _fallback: Decimal) -> Decimal {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_safe_divide() {
        assert_eq!(safe_divide(dec!(10.0), dec!(2.0), dec!(0.0)), dec!(5.0));
        assert_eq!(safe_divide(dec!(10.0), dec!(0.0), dec!(0.0)), dec!(0.0));
        assert_eq!(safe_divide(dec!(10.0), dec!(0.0), dec!(-1.0)), dec!(-1.0));
        assert_eq!(safe_divide(dec!(0.0), dec!(0.0), dec!(0.0)), dec!(0.0));
    }

    #[test]
    fn test_safe_finite() {
        assert_eq!(safe_finite(dec!(10.0), dec!(0.0)), dec!(10.0));
    }
}
