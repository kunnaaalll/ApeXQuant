#![allow(warnings, clippy::all, deprecated)]
use crate::confidence::{ConfidencePenalty, ConfidenceScore, SampleQuality, SampleQualityGrade};
use rust_decimal::Decimal;

#[test]
fn test_sample_quality_grades() {
    let q1 = SampleQuality::new(15);
    assert_eq!(q1.grade(), SampleQualityGrade::Insufficient);

    let q2 = SampleQuality::new(30);
    assert_eq!(q2.grade(), SampleQualityGrade::Insufficient); // Wait, requirements were: 20, 50, 100, 300, 1000. Wait, >= 20 is Insufficient? Let's check requirements.
                                                              // "Thresholds: 20, 50, 100, 300, 1000. Grades: Insufficient, Weak, Adequate, Strong, InstitutionalGrade"
                                                              // So < 50 = Insufficient, >= 50 = Weak, >= 100 = Adequate, >= 300 = Strong, >= 1000 = InstitutionalGrade.
                                                              // Oh, but < 20 is also Insufficient.
}

#[test]
fn test_confidence_penalty() {
    let drawdown = Decimal::new(10, 0); // 10
    let variance = Decimal::new(5, 0); // 5
    let instability = Decimal::new(2, 0); // 2
    let edge_decay = Decimal::new(3, 0); // 3
    let consecutive_losses = 4; // 4 * 0.5 = 2.0

    let penalty = ConfidencePenalty::calculate(
        drawdown,
        variance,
        instability,
        edge_decay,
        consecutive_losses,
    );
    assert_eq!(penalty.amount, Decimal::new(22, 0)); // 10 + 5 + 2 + 3 + 2 = 22
}

#[test]
fn test_confidence_calculate() {
    let sample_quality = Decimal::new(80, 0);
    let edge_score = Decimal::new(80, 0);
    let stability = Decimal::new(80, 0);
    // Base = 80

    let penalty = ConfidencePenalty {
        amount: Decimal::new(20, 0),
    };

    let score = ConfidenceScore::calculate(sample_quality, edge_score, stability, penalty);
    // 80 - 20 = 60
    assert_eq!(score.value(), Decimal::new(60, 0));
}
