#![allow(warnings, clippy::all, deprecated)]
use crate::adequacy::{AdequacyGrade, SampleAdequacy};
use crate::confidence::context::{ContextConfidenceScore, ContextConfidenceTier};
use crate::pattern::{PatternAssessment, PatternGrade};
use crate::ranking::context::{ContextRank, ContextRankTier};
use crate::regime::{RegimeAssessment, RegimeGrade};
use crate::session::{SessionAssessment, SessionGrade};
use crate::symbol::{SymbolAssessment, SymbolGrade};
use crate::timeframe::{TimeframeAssessment, TimeframeGrade};
use rust_decimal::Decimal;

#[test]
fn test_regime_grading() {
    let mut assessment = RegimeAssessment {
        expectancy: Decimal::from(2),
        edge: Decimal::from(2),
        confidence: Decimal::from(2),
        stability: Decimal::from(2),
        drawdown: Decimal::from(0), // Tests division by zero fallback to 1
        health: Decimal::from(50),
    };

    // 2 * 2 * 2 * 2 * 50 = 800 (Strong)
    assert_eq!(assessment.grade(), RegimeGrade::Strong);

    assessment.health = Decimal::from(100);
    // 2 * 2 * 2 * 2 * 100 = 1600 (Elite)
    assert_eq!(assessment.grade(), RegimeGrade::Elite);

    assessment.drawdown = Decimal::from(1000);
    // 1600 / 1000 = 1.6 (Forbidden, since < 10)
    assert_eq!(assessment.grade(), RegimeGrade::Forbidden);
}

#[test]
fn test_session_grading() {
    let assessment = SessionAssessment {
        win_rate: Decimal::from(5),
        expectancy: Decimal::from(5),
        edge: Decimal::from(5),
        confidence: Decimal::from(2),
        degradation: Decimal::from(50),
    };

    // (5*5*5*2) - 50 = 250 - 50 = 200 (Normal)
    assert_eq!(assessment.grade(), SessionGrade::Normal);
}

#[test]
fn test_symbol_penalties() {
    let mut assessment = SymbolAssessment {
        expectancy: Decimal::from(10),
        stability: Decimal::from(10),
        drawdown: Decimal::from(1),
        confidence: Decimal::from(10),
        sample_count: 200,
    };

    // Base: 10 * 10 * 10 = 1000 (Elite)
    assert_eq!(assessment.grade(), SymbolGrade::Elite);

    // Test < 100 penalty (0.8x)
    assessment.sample_count = 80;
    // 1000 * 0.8 = 800 (Strong)
    assert_eq!(assessment.grade(), SymbolGrade::Strong);

    // Test < 50 penalty (0.5x)
    assessment.sample_count = 40;
    // 1000 * 0.5 = 500 (Strong)
    assert_eq!(assessment.grade(), SymbolGrade::Strong);

    // Test < 20 penalty (0.1x)
    assessment.sample_count = 10;
    // 1000 * 0.1 = 100 (Normal)
    assert_eq!(assessment.grade(), SymbolGrade::Normal);
}

#[test]
fn test_adequacy_grading() {
    let adeq = SampleAdequacy::new(1000);
    assert_eq!(adeq.grade(), AdequacyGrade::InstitutionalGrade);
    assert_eq!(adeq.confidence_penalty(), Decimal::new(0, 0));

    let adeq2 = SampleAdequacy::new(15);
    assert_eq!(adeq2.grade(), AdequacyGrade::Insufficient);
    assert_eq!(adeq2.confidence_penalty(), Decimal::new(90, 2));
}

#[test]
fn test_confidence_clamping() {
    let conf = ContextConfidenceScore::calculate(
        Decimal::from(100),
        Decimal::from(100),
        Decimal::from(0),
        Decimal::from(0),
    );
    assert_eq!(conf.value(), Decimal::from(100));
    assert_eq!(conf.tier(), ContextConfidenceTier::VeryHigh);

    // Check downward bounds clamping
    let conf2 = ContextConfidenceScore::calculate(
        Decimal::from(10),
        Decimal::from(10),
        Decimal::from(100),
        Decimal::from(100),
    );
    assert_eq!(conf2.value(), Decimal::from(0));
}

#[test]
fn test_context_ranking() {
    let rank = ContextRank::calculate(
        Decimal::from(10),
        Decimal::from(10),
        Decimal::from(10),
        Decimal::from(0),
    );
    // 10 * 10 * 10 = 1000 => Elite
    assert_eq!(rank.tier, ContextRankTier::Elite);
}

#[test]
fn test_determinism_loop() {
    let mut score = Decimal::from(1);
    let increment = Decimal::new(1, 4); // 0.0001

    for _ in 0..100_000 {
        score += increment;
    }

    assert_eq!(score, Decimal::from(11));
}

#[test]
fn test_timeframe_grading() {
    let assessment = TimeframeAssessment {
        expectancy: Decimal::from(5),
        confidence: Decimal::from(5),
        stability: Decimal::from(5),
    };
    // 5 * 5 * 5 = 125 => Normal
    assert_eq!(assessment.grade(), TimeframeGrade::Normal);
}

#[test]
fn test_pattern_grading() {
    let assessment = PatternAssessment {
        setup_expectancy: Decimal::from(10),
        rr: Decimal::from(2),
        confidence: Decimal::from(10),
        sample_quality: Decimal::from(5),
    };
    // 10 * 2 * 10 * 5 = 1000 => Exceptional
    assert_eq!(assessment.grade(), PatternGrade::Exceptional);
}
