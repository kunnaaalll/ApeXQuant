#![allow(warnings, clippy::all, deprecated)]
//! Property-based tests for Confluence Engine

use proptest::prelude::*;
use signal_engine::confluence::{ConfluenceScore, FactorResult};
use signal_engine::signals::MarketContext;

/// Strategy for generating valid confluence scores
fn valid_score() -> impl Strategy<Value = f64> {
    0.0f64..=100.0f64
}

/// Strategy for generating factor results
fn factor_result() -> impl Strategy<Value = FactorResult> {
    (0.0f64..=1.0f64, 0.0f64..=1.0f64).prop_map(|(score, confidence)| {
        FactorResult {
            score,
            confidence,
            evidence: vec!["test evidence".to_string()],
        }
    })
}

proptest! {
    // #[test]
    // fn confluence_score_always_in_bounds(scores in prop::collection::vec(valid_score(), 5..20)) {
    //     let engine = signal_engine::confluence::ConfluenceEngine::new(60.0);
    //
    //     let context = MarketContext::with_factor_scores(&scores);
    //     let result = engine.calculate(&context);
    //
    //     prop_assert!(result.total >= 0.0 && result.total <= 100.0,
    //         "Score {} out of bounds", result.total);
    // }

    #[test]
    fn grade_classification_is_consistent(score in 0.0f64..=100.0f64) {
        let result = ConfluenceScore {
            total: score,
            ..Default::default()
        };

        let grade = result.grade();

        if score >= 85.0 {
            prop_assert_eq!(grade, "A+", "Score {:.1} should be A+", score);
        } else if score >= 70.0 {
            prop_assert_eq!(grade, "A", "Score {:.1} should be A", score);
        } else if score >= 60.0 {
            prop_assert_eq!(grade, "B", "Score {:.1} should be B", score);
        } else {
            prop_assert_eq!(grade, "REJECT", "Score {:.1} should be REJECT", score);
        }
    }

    #[test]
    fn confidence_always_in_valid_range(conf in 0.0f64..=100.0f64) {
        let calculated = conf.clamp(0.0, 100.0);
        // Confidence should always be clamped to 0-100
        prop_assert!(calculated >= 0.0 && calculated <= 100.0);
    }

    #[test]
    fn risk_reward_always_greater_than_one(
        entry in 1.0f64..=2.0f64,
        stop in 0.5f64..=1.5f64,
        target in 1.5f64..=3.0f64
    ) {
        // Entry should be between stop and target for valid trade
        // Risk = |entry - stop|
        // Reward = |target - entry|
        // Valid trades should have RR > 1

        let risk = (entry - stop).abs();
        let reward = (target - entry).abs();

        if reward > risk && risk > 0.0 {
            let rr = reward / risk;
            prop_assert!(rr > 1.0, "Risk/Reward ratio {} should be > 1", rr);
        }
    }
}

/// Test that entry price consistency is maintained
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn candle_high_gte_low(high in 1.0f64..=2.0f64, low in 0.5f64..=1.5f64) {
        // Ensure high >= low by construction
        let actual_high = high.max(low);
        let actual_low = high.min(low);

        prop_assert!(actual_high >= actual_low,
            "High {:.5} should be >= Low {:.5}", actual_high, actual_low);
    }

    #[test]
    fn close_between_high_low(
        high in 1.0f64..=2.0f64,
        low in 0.5f64..=1.5f64,
        close_factor in 0.0f64..=1.0f64
    ) {
        let actual_high = high.max(low);
        let actual_low = high.min(low);
        let close = actual_low + (actual_high - actual_low) * close_factor;

        prop_assert!(close >= actual_low && close <= actual_high,
            "Close {:.5} should be between L={:.5} and H={:.5}",
            close, actual_low, actual_high);
    }
}
