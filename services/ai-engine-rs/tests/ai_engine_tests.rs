use ai_engine_rs::feature_engineering::{FeatureEngine, MarketFeatureSet};
use ai_engine_rs::regime_classifier::{RegimeClassifier, MarketRegimeType};
use ai_engine_rs::pattern_recognition::{PatternRecognizer, PatternType};
use ai_engine_rs::embeddings::{EmbeddingsEngine, EmbeddingVector};
use rust_decimal::Decimal;

#[test]
fn test_feature_engineering_deterministic() {
    let engine = FeatureEngine::new();
    let data = vec![Decimal::new(100, 0), Decimal::new(105, 0), Decimal::new(95, 0), Decimal::new(102, 0)];
    let features = engine.compute_features(&data);
    
    assert_eq!(features.price_action.open, Decimal::new(100, 0));
    assert_eq!(features.price_action.high, Decimal::new(105, 0));
    assert_eq!(features.price_action.low, Decimal::new(95, 0));
    assert_eq!(features.price_action.close, Decimal::new(102, 0));
}

#[test]
fn test_regime_classifier() {
    let classifier = RegimeClassifier::new();
    let regime = classifier.classify(Decimal::new(6, 1), Decimal::new(1, 0), Decimal::new(5000, 0)); // Price change 0.6, vol 1.0, vol 5000
    
    // Trend threshold is 0.5, so price change 0.6 should be TrendingUp
    assert_eq!(regime.regime_type, MarketRegimeType::TrendingUp);
}

#[test]
fn test_pattern_recognition() {
    let recognizer = PatternRecognizer::new();
    let highs = vec![Decimal::new(105, 0), Decimal::new(103, 0), Decimal::new(106, 0)];
    let lows = vec![Decimal::new(100, 0), Decimal::new(101, 0), Decimal::new(104, 0)];
    let times = vec![1, 2, 3];
    
    let fvgs = recognizer.detect_fvg(&highs, &lows, &times);
    // Low of candle 3 is 104, High of candle 1 is 105. No Bullish FVG.
    // High of candle 3 is 106, Low of candle 1 is 100. No Bearish FVG.
    assert_eq!(fvgs.len(), 0);
    
    let highs2 = vec![Decimal::new(100, 0), Decimal::new(95, 0), Decimal::new(90, 0)];
    let lows2 = vec![Decimal::new(90, 0), Decimal::new(85, 0), Decimal::new(80, 0)];
    let times2 = vec![1, 2, 3];
    // High 3 = 90, Low 1 = 90. No FVG.
    let fvgs2 = recognizer.detect_fvg(&highs2, &lows2, &times2);
    assert_eq!(fvgs2.len(), 0);
}

#[test]
fn test_embeddings_engine() {
    let engine = EmbeddingsEngine::new();
    let emb1 = engine.generate_embedding("strategy_A");
    let emb2 = engine.generate_embedding("strategy_A");
    let emb3 = engine.generate_embedding("completely_different_string");
    
    // Identical string -> identical embedding -> similarity 1.0
    assert!((emb1.cosine_similarity(&emb2) - 1.0).abs() < 0.001);
    
    // Different string -> different embedding
    assert!((emb1.cosine_similarity(&emb3) - 1.0).abs() > 0.0001);
}
