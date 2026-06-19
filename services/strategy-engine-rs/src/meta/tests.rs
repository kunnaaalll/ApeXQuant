use super::strategy_health::StrategyHealth;
use super::strategy_state::StrategyState;
use rust_decimal::Decimal;

#[test]
fn test_strategy_state_transitions() {
    let state = StrategyState::Normal;
    assert_eq!(state, StrategyState::Normal);
    
    let new_state = StrategyState::Elite;
    assert_eq!(new_state, StrategyState::Elite);
}

#[test]
fn test_health_recovery_limit() {
    let mut health = StrategyHealth::new(50);
    
    // Attempt to recover by 10
    health.recover(Decimal::from(10));
    
    // Limit is +5 per cycle
    assert_eq!(health.score(), Decimal::from(55));
}
