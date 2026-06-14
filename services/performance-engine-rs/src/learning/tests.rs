#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use crate::learning::*;

    #[test]
    fn test_evidence_accumulator() {
        let mut acc = EvidenceAccumulator::new(dec!(0.9));
        acc.record_success(dec!(10.0));
        assert_eq!(acc.successful_conditions_score, dec!(10.0));
        
        acc.record_success(dec!(5.0));
        // 10.0 * 0.9 + 5.0 = 14.0
        assert_eq!(acc.successful_conditions_score, dec!(14.0));
    }

    #[test]
    fn test_adaptive_weights() {
        let mut weights = AdaptiveWeights::new(dec!(0.5), dec!(0.5), dec!(0.05));
        weights.update_weights(true);
        assert_eq!(weights.recent_weight, dec!(0.55));
        assert_eq!(weights.historical_weight, dec!(0.45));
    }
}
