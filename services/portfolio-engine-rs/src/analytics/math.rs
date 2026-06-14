// src/analytics/math.rs

/// Safe division that returns a fallback value if the denominator is 0.0 or if the result is NaN or infinite.
/// By default, the fallback is 0.0, but it can be overridden if needed.
#[inline]
pub fn safe_divide(numerator: f64, denominator: f64, fallback: f64) -> f64 {
    if denominator.abs() < f64::EPSILON {
        return fallback;
    }
    let result = numerator / denominator;
    if result.is_nan() || result.is_infinite() {
        return fallback;
    }
    result
}

/// Ensures a value is strictly finite and bounded, reverting to fallback otherwise.
#[inline]
pub fn safe_finite(value: f64, fallback: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        return fallback;
    }
    value
}

/// Clamps a value between a min and max safely, handling NaNs
#[inline]
pub fn safe_clamp(value: f64, min: f64, max: f64, fallback: f64) -> f64 {
    if value.is_nan() {
        return fallback;
    }
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

    #[test]
    fn test_safe_divide() {
        assert_eq!(safe_divide(10.0, 2.0, 0.0), 5.0);
        assert_eq!(safe_divide(10.0, 0.0, 0.0), 0.0);
        assert_eq!(safe_divide(10.0, 0.0, -1.0), -1.0);
        assert_eq!(safe_divide(0.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn test_safe_finite() {
        assert_eq!(safe_finite(10.0, 0.0), 10.0);
        assert_eq!(safe_finite(f64::NAN, 0.0), 0.0);
        assert_eq!(safe_finite(f64::INFINITY, 0.0), 0.0);
        assert_eq!(safe_finite(f64::NEG_INFINITY, 0.0), 0.0);
    }
}
