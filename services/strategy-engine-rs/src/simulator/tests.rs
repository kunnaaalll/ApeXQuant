use super::variant_runner::VariantComparison;
use super::replay_engine::ReplayEngine;

#[test]
fn test_variant_selection() {
    let comparison = VariantComparison::new("VariantA".to_string(), "VariantB".to_string());
    assert_eq!(comparison.best_variant, "VariantA");
    assert_eq!(comparison.worst_variant, "VariantB");
}

#[test]
fn test_replay() {
    let engine = ReplayEngine::new();
    assert!(engine.active, "ReplayEngine should be active upon initialization");
}
