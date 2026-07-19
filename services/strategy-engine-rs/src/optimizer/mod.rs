pub mod pattern_optimizer;
pub mod regime_optimizer;
pub mod session_optimizer;
pub mod symbol_optimizer;
pub mod timeframe_optimizer;

#[cfg(test)]
mod tests;

pub use pattern_optimizer::PatternOptimizer;
pub use regime_optimizer::RegimeOptimizer;
pub use session_optimizer::SessionOptimizer;
pub use symbol_optimizer::SymbolOptimizer;
pub use timeframe_optimizer::TimeframeOptimizer;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RankingGrade {
    Forbidden = 0,
    Weak = 1,
    Normal = 2,
    Strong = 3,
    Elite = 4,
}

pub(crate) fn calculate_score(
    expectancy: Decimal,
    confidence: Decimal,
    stability: Decimal,
    drawdown: Decimal,
    sample_quality: Decimal,
) -> Decimal {
    let zero = dec!(0.0);
    let one_hundred = dec!(100.0);
    let epsilon = dec!(0.0001);

    // Protect against division by zero
    let safe_drawdown = if drawdown.abs() < epsilon {
        epsilon
    } else {
        drawdown.abs()
    };

    // Protect against underflow/overflow - use checked math or safe bounds
    // Score formula: (Expectancy * Confidence * Stability * SampleQuality) / Drawdown
    // Normalize to 0-100. Assume inputs are generally in 0-1 range except Expectancy which can be anything.

    // We construct a scoring function that naturally bounds or we clamp it.
    // We add 1.0 to expectancy in case it's negative to penalize but not crash,
    // actually let's just clamp the intermediate parts safely.

    // Normalize expectancy to positive domain for score
    let exp_factor = if expectancy < zero { zero } else { expectancy };

    // Avoid overflow by multiplying step by step and clamping
    let p1 = exp_factor.checked_mul(confidence).unwrap_or(zero);
    let p2 = p1.checked_mul(stability).unwrap_or(zero);
    let p3 = p2.checked_mul(sample_quality).unwrap_or(zero);

    let raw_score = p3.checked_div(safe_drawdown).unwrap_or(zero);

    // Scale to a realistic 0-100 range. We multiply by 100 and clamp.
    let scaled = raw_score.checked_mul(one_hundred).unwrap_or(zero);

    if scaled > one_hundred {
        one_hundred
    } else if scaled < zero {
        zero
    } else {
        scaled
    }
}

pub(crate) fn grade_from_score(score: Decimal) -> RankingGrade {
    if score >= dec!(80.0) {
        RankingGrade::Elite
    } else if score >= dec!(60.0) {
        RankingGrade::Strong
    } else if score >= dec!(40.0) {
        RankingGrade::Normal
    } else if score >= dec!(20.0) {
        RankingGrade::Weak
    } else {
        RankingGrade::Forbidden
    }
}
