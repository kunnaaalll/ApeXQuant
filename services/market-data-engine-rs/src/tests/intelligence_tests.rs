use rust_decimal::Decimal;
use crate::spread::{SpreadEngine, SpreadGrade};
use crate::volatility::{VolatilityEngine, VolatilityGrade};
use crate::depth::{DepthEngine, DepthGrade};
use crate::imbalance::{ImbalanceEngine, ImbalanceGrade};
use crate::market_state::{MarketStateEngine, MarketState};

#[test]
fn test_determinism_100k_iterations() -> Result<(), &'static str> {
    let bid = Decimal::from(100);
    let ask = Decimal::new(1001, 1); // 100.1

    for _ in 0..100_000 {
        let spread_metrics = SpreadEngine::calculate(bid, ask)?;
        assert_eq!(spread_metrics.grade, SpreadGrade::Normal);
        
        let vol_metrics = VolatilityEngine::calculate(Decimal::from(105), Decimal::from(95), Decimal::from(100))?;
        assert_eq!(vol_metrics.grade, VolatilityGrade::Extreme); // 10% movement -> 1000 bps -> extreme
        
        let depth_metrics = DepthEngine::evaluate(Decimal::from(50), Decimal::from(50), 10)?;
        assert_eq!(depth_metrics.grade, DepthGrade::Normal);
    }
    Ok(())
}

#[test]
fn test_division_by_zero_protection() -> Result<(), &'static str> {
    // Imbalance total 0
    let imbalance = ImbalanceEngine::calculate(Decimal::ZERO, Decimal::ZERO)?;
    assert_eq!(imbalance.grade, ImbalanceGrade::Balanced);
    
    // Spread ask 0
    let spread = SpreadEngine::calculate(Decimal::ZERO, Decimal::ZERO);
    assert!(spread.is_err());
    Ok(())
}

#[test]
fn test_state_transitions() {
    assert!(MarketStateEngine::transition(MarketState::Broken, MarketState::Healthy).is_err());
    assert!(MarketStateEngine::transition(MarketState::Healthy, MarketState::Warning).is_ok());
}
