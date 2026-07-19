use rust_decimal::Decimal;

use super::decay::HeatDecayModel;
use super::heat_score::{HeatContributionBreakdown, PortfolioHeat, PortfolioHeatState};
use super::risk_budget::RiskBudget;

#[test]
fn test_heat_score_bounding() {
    let breakdown = HeatContributionBreakdown {
        factors: vec![],
        total_score: 150,
    };

    let heat = PortfolioHeat::new(150, breakdown);
    assert_eq!(heat.score, 100);
    assert_eq!(heat.state, PortfolioHeatState::Frozen);
}

#[test]
fn test_heat_decay() {
    let breakdown = HeatContributionBreakdown {
        factors: vec![],
        total_score: 85,
    };

    let mut heat = PortfolioHeat::new(85, breakdown);
    assert_eq!(heat.state, PortfolioHeatState::Critical);

    heat.apply_decay(10);
    assert_eq!(heat.score, 75);
    assert_eq!(heat.state, PortfolioHeatState::Hot);

    heat.apply_decay(100); // Should saturate at 0
    assert_eq!(heat.score, 0);
    assert_eq!(heat.state, PortfolioHeatState::Cold);
}

#[test]
fn test_decay_model_cooldown() {
    let mut model = HeatDecayModel::new(5, 3);

    // Register loss, resets ticks
    model.register_loss();
    assert_eq!(model.calculate_decay(), 0);

    model.register_tick(); // tick 1
    assert_eq!(model.calculate_decay(), 0);

    model.register_tick(); // tick 2
    assert_eq!(model.calculate_decay(), 0);

    model.register_tick(); // tick 3
    assert_eq!(model.calculate_decay(), 5);
}

#[test]
fn test_risk_budget_allocations() {
    let budget = RiskBudget::new(
        Decimal::new(1000, 1),
        Decimal::new(500, 1),
        Decimal::new(100, 1),
        Decimal::new(200, 1),
    );

    assert_eq!(budget.total_risk_capacity, Decimal::new(1000, 1));
    assert_eq!(budget.remaining_risk, Decimal::new(200, 1)); // 100 - 50 - 10 - 20 = 20

    assert!(budget.can_allocate(Decimal::new(150, 1)));
    assert!(!budget.can_allocate(Decimal::new(250, 1)));
}
