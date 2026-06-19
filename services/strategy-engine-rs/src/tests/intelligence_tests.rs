use rust_decimal::Decimal;
use crate::intelligence::{EdgeIntelligence, PatternAssessment, Recommendation};

#[test]
fn test_edge_intelligence_assess() {
    // Exceptional: >= 0.5 expectancy AND >= 0.8 stability
    let exceptional = EdgeIntelligence::new(
        Decimal::new(6, 1), // 0.6
        Decimal::new(6, 1),
        Decimal::new(2, 0),
        Decimal::new(9, 1), // 0.9
        Decimal::new(1, 1),
    );
    assert_eq!(exceptional.assess(), PatternAssessment::Exceptional);

    // Strong: >= 0.2 expectancy
    let strong = EdgeIntelligence::new(
        Decimal::new(3, 1), // 0.3
        Decimal::new(5, 1),
        Decimal::new(15, 1),
        Decimal::new(5, 1), // 0.5 (fails exceptional stability)
        Decimal::new(2, 1),
    );
    assert_eq!(strong.assess(), PatternAssessment::Strong);

    // Normal: > 0.0 expectancy
    let normal = EdgeIntelligence::new(
        Decimal::new(1, 1), // 0.1
        Decimal::new(4, 1),
        Decimal::new(1, 0),
        Decimal::new(5, 1),
        Decimal::new(3, 1),
    );
    assert_eq!(normal.assess(), PatternAssessment::Normal);

    // Weak: > -0.2 expectancy
    let weak = EdgeIntelligence::new(
        Decimal::new(-1, 1), // -0.1
        Decimal::new(3, 1),
        Decimal::new(8, 1),
        Decimal::new(4, 1),
        Decimal::new(4, 1),
    );
    assert_eq!(weak.assess(), PatternAssessment::Weak);

    // Negative: <= -0.2 expectancy
    let negative = EdgeIntelligence::new(
        Decimal::new(-3, 1), // -0.3
        Decimal::new(2, 1),
        Decimal::new(5, 1),
        Decimal::new(2, 1),
        Decimal::new(5, 1),
    );
    assert_eq!(negative.assess(), PatternAssessment::Negative);
}

#[test]
fn test_edge_intelligence_recommend() {
    let exceptional = EdgeIntelligence::new(
        Decimal::new(6, 1), Decimal::new(6, 1), Decimal::new(2, 0), Decimal::new(9, 1), Decimal::new(1, 1),
    );
    assert_eq!(exceptional.recommend(), Recommendation::IncreaseAllocation);

    let negative = EdgeIntelligence::new(
        Decimal::new(-3, 1), Decimal::new(2, 1), Decimal::new(5, 1), Decimal::new(2, 1), Decimal::new(5, 1),
    );
    assert_eq!(negative.recommend(), Recommendation::Pause);
}
