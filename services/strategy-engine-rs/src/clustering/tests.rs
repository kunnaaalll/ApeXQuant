use super::*;
use rust_decimal_macros::dec;

#[test]
fn test_cluster_confidence_bounds() {
    let mut engine = ClusterEngine::new();

    // Normal
    engine.update(ClusterType::Momentum, dec!(80.0));
    assert_eq!(engine.state().active_cluster, ClusterType::Momentum);
    assert_eq!(engine.state().confidence, dec!(80.0));

    // Over bounds
    engine.update(ClusterType::RiskOn, dec!(150.0));
    assert_eq!(engine.state().confidence, dec!(100.0));

    // Under bounds
    engine.update(ClusterType::RiskOff, dec!(-50.0));
    assert_eq!(engine.state().confidence, dec!(0.0));
}
