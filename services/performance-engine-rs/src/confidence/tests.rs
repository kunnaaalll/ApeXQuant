use crate::confidence::*;
#[cfg(test)]
use rust_decimal_macros::dec;

#[test]
fn test_sample_quality() {
    let sq = SampleQuality::evaluate(10);
    assert_eq!(sq.state, SampleQualityState::Insufficient);
    assert_eq!(sq.confidence_multiplier, dec!(0.1));

    let sq2 = SampleQuality::evaluate(45);
    assert_eq!(sq2.state, SampleQualityState::Weak);
    assert_eq!(sq2.confidence_multiplier, dec!(0.5));
}

#[test]
fn test_penalties() {
    let p1 = ConfidencePenalty::small_sample_size(15);
    assert!(p1.is_some());
    assert_eq!(p1.unwrap().severity, PenaltySeverity::Critical);

    let p2 = ConfidencePenalty::large_drawdown(dec!(0.25));
    assert!(p2.is_some());

    let p3 = ConfidencePenalty::consecutive_losses(6);
    assert!(p3.is_some());
}

#[test]
fn test_score_calculation_determinism() {
    let penalties = vec![
        ConfidencePenalty::large_drawdown(dec!(0.25)).unwrap(),
        ConfidencePenalty::consecutive_losses(6).unwrap(),
    ];

    let mut prev_score: Option<ConfidenceScore> = None;
    for _ in 0..100_000 {
        let score = ConfidenceScore::calculate(
            dec!(80),
            dec!(80),
            dec!(100),
            dec!(70),
            dec!(60),
            dec!(50),
            dec!(50),
            dec!(80),
            dec!(90),
            penalties.clone(),
        );

        if let Some(prev) = &prev_score {
            assert_eq!(score.final_score, prev.final_score);
        }
        prev_score = Some(score);
    }
}
