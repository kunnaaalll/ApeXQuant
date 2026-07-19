use crate::quality::quality_score::{
    PortfolioQuality, PortfolioQualityBreakdown, PortfolioQualityState, QualityContribution,
    QualityEvent,
};
use rust_decimal::Decimal;

fn default_breakdown() -> PortfolioQualityBreakdown {
    let contrib = QualityContribution {
        weight: Decimal::ZERO,
        score: Decimal::ZERO,
        reason: "".to_string(),
    };
    PortfolioQualityBreakdown {
        win_rate: contrib.clone(),
        profit_factor: contrib.clone(),
        expectancy: contrib.clone(),
        average_rr: contrib.clone(),
        position_quality: contrib.clone(),
        position_health: contrib.clone(),
        capital_efficiency: contrib.clone(),
        trade_efficiency: contrib.clone(),
        holding_efficiency: contrib.clone(),
        allocation_efficiency: contrib.clone(),
        recovery_factor: contrib.clone(),
        recent_performance: contrib.clone(),
        drawdown_efficiency: contrib,
    }
}

#[test]
fn test_quality_initialization() {
    let quality = PortfolioQuality::new(1000);
    assert_eq!(quality.current_score, Decimal::new(50, 0));
    assert_eq!(quality.state, PortfolioQualityState::Neutral);
    assert_eq!(quality.version, 1);
}

#[test]
fn test_quality_state_boundaries() {
    assert_eq!(
        PortfolioQuality::determine_state(Decimal::new(95, 0)),
        PortfolioQualityState::Excellent
    );
    assert_eq!(
        PortfolioQuality::determine_state(Decimal::new(90, 0)),
        PortfolioQualityState::Excellent
    );
    assert_eq!(
        PortfolioQuality::determine_state(Decimal::new(899, 1)),
        PortfolioQualityState::Good
    );
    assert_eq!(
        PortfolioQuality::determine_state(Decimal::new(75, 0)),
        PortfolioQualityState::Good
    );
    assert_eq!(
        PortfolioQuality::determine_state(Decimal::new(749, 1)),
        PortfolioQualityState::Neutral
    );
    assert_eq!(
        PortfolioQuality::determine_state(Decimal::new(50, 0)),
        PortfolioQualityState::Neutral
    );
    assert_eq!(
        PortfolioQuality::determine_state(Decimal::new(499, 1)),
        PortfolioQualityState::Weak
    );
    assert_eq!(
        PortfolioQuality::determine_state(Decimal::new(25, 0)),
        PortfolioQualityState::Weak
    );
    assert_eq!(
        PortfolioQuality::determine_state(Decimal::new(249, 1)),
        PortfolioQualityState::Critical
    );
    assert_eq!(
        PortfolioQuality::determine_state(Decimal::ZERO),
        PortfolioQualityState::Critical
    );
}

#[test]
fn test_apply_event_clamping() {
    let mut quality = PortfolioQuality::new(1000);
    let breakdown = default_breakdown();

    let snapshot_high = quality.apply_event(
        QualityEvent::PnLChanged,
        Decimal::new(150, 0),
        breakdown.clone(),
        1001,
    );
    assert_eq!(snapshot_high.composite_score, Decimal::new(100, 0));
    assert_eq!(snapshot_high.state, PortfolioQualityState::Excellent);

    let snapshot_low = quality.apply_event(
        QualityEvent::PnLChanged,
        Decimal::new(-50, 0),
        breakdown,
        1002,
    );
    assert_eq!(snapshot_low.composite_score, Decimal::ZERO);
    assert_eq!(snapshot_low.state, PortfolioQualityState::Critical);
}

#[test]
fn test_apply_decay() {
    let mut quality = PortfolioQuality::new(1000);
    let breakdown = default_breakdown();
    quality.apply_event(
        QualityEvent::PnLChanged,
        Decimal::new(80, 0),
        breakdown,
        1001,
    ); // State is Good

    // Apply 10 points decay
    let snapshot = quality.apply_decay(Decimal::new(10, 0), 1002);
    assert_eq!(snapshot.composite_score, Decimal::new(70, 0));
    assert_eq!(snapshot.state, PortfolioQualityState::Neutral); // Dropped to Neutral
    assert_eq!(snapshot.version, 3);
}
