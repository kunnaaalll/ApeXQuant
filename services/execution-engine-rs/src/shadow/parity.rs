use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityLevel {
    Perfect,
    Excellent,
    Good,
    Acceptable,
    Poor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityScore {
    pub value: Decimal,
    pub level: ParityLevel,
}

pub struct ParityEngine;

impl ParityEngine {
    pub fn compute(stats: &crate::shadow::statistics::ShadowStatistics) -> ParityScore {
        let total = stats.total_matches();
        if total == 0 {
            return ParityScore {
                value: dec!(100),
                level: ParityLevel::Perfect,
            };
        }

        let total_dec = Decimal::from(total);
        let _exact_dec = Decimal::from(stats.exact_match_count);
        let close_dec = Decimal::from(stats.close_match_count);
        let warning_dec = Decimal::from(stats.warning_count);
        let mismatch_dec = Decimal::from(stats.mismatch_count);
        let critical_dec = Decimal::from(stats.critical_mismatch_count);

        let mut score = dec!(100.0);

        let exact_penalty = dec!(0.0);
        let close_penalty = (close_dec / total_dec) * dec!(5.0);
        let warning_penalty = (warning_dec / total_dec) * dec!(15.0);
        let mismatch_penalty = (mismatch_dec / total_dec) * dec!(40.0);
        let critical_penalty = (critical_dec / total_dec) * dec!(100.0);

        let total_penalty =
            exact_penalty + close_penalty + warning_penalty + mismatch_penalty + critical_penalty;

        score -= total_penalty;

        if score < dec!(0.0) {
            score = dec!(0.0);
        } else if score > dec!(100.0) {
            score = dec!(100.0);
        }

        let level = if score == dec!(100.0) {
            ParityLevel::Perfect
        } else if score >= dec!(95.0) {
            ParityLevel::Excellent
        } else if score >= dec!(85.0) {
            ParityLevel::Good
        } else if score >= dec!(70.0) {
            ParityLevel::Acceptable
        } else {
            ParityLevel::Poor
        };

        ParityScore {
            value: score,
            level,
        }
    }
}
