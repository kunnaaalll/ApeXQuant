use super::block::{BlockEngine, BlockOutcome};
use super::close::{CloseExposureEngine, CloseOutcome};
use super::consistency::RecommendationConsistencyValidator;
use super::increase::{IncreaseExposureEngine, IncreaseOutcome};
use super::reduce::{ReduceExposureEngine, ReduceOutcome};

#[test]
fn test_frozen_portfolio_recommendations() {
    let increase = IncreaseExposureEngine::evaluate(0, 100, 100, true, false);
    let reduce = ReduceExposureEngine::evaluate(0, 100, true, false);
    let close = CloseExposureEngine::evaluate(0, 100, true, false);
    let block = BlockEngine::evaluate(0, 100, true, false);

    assert_eq!(increase.outcome, IncreaseOutcome::Reject);
    assert_eq!(reduce.outcome, ReduceOutcome::EmergencyReduction);
    assert_eq!(close.outcome, CloseOutcome::EmergencyLiquidation);
    assert_eq!(block.outcome, BlockOutcome::Freeze);

    let consistency = RecommendationConsistencyValidator::validate(
        &increase.outcome,
        &reduce.outcome,
        &close.outcome,
        &block.outcome,
        false,
        true,
    );

    assert!(consistency.is_ok());
}

#[test]
fn test_critical_drawdown_recommendations() {
    let increase = IncreaseExposureEngine::evaluate(0, 100, 100, false, true);
    let reduce = ReduceExposureEngine::evaluate(0, 100, false, true);
    let close = CloseExposureEngine::evaluate(0, 100, false, true);
    let block = BlockEngine::evaluate(0, 100, false, true);

    assert_eq!(increase.outcome, IncreaseOutcome::Reject);
    assert_eq!(reduce.outcome, ReduceOutcome::EmergencyReduction);
    assert_eq!(close.outcome, CloseOutcome::EmergencyLiquidation);
    assert_eq!(block.outcome, BlockOutcome::Block);

    let consistency = RecommendationConsistencyValidator::validate(
        &increase.outcome,
        &reduce.outcome,
        &close.outcome,
        &block.outcome,
        true,
        false,
    );

    assert!(consistency.is_ok());
}

#[test]
fn test_healthy_portfolio_recommendations() {
    let increase = IncreaseExposureEngine::evaluate(20, 95, 95, false, false);
    let reduce = ReduceExposureEngine::evaluate(20, 95, false, false);
    let close = CloseExposureEngine::evaluate(20, 95, false, false);
    let block = BlockEngine::evaluate(20, 95, false, false);

    assert_eq!(increase.outcome, IncreaseOutcome::Increase);
    assert_eq!(reduce.outcome, ReduceOutcome::NoAction);
    assert_eq!(close.outcome, CloseOutcome::Hold);
    assert_eq!(block.outcome, BlockOutcome::Allow);

    let consistency = RecommendationConsistencyValidator::validate(
        &increase.outcome,
        &reduce.outcome,
        &close.outcome,
        &block.outcome,
        false,
        false,
    );

    assert!(consistency.is_ok());
}

#[test]
fn test_consistency_validator_contradiction_frozen_increase() {
    let err = RecommendationConsistencyValidator::validate(
        &IncreaseOutcome::Increase,
        &ReduceOutcome::NoAction,
        &CloseOutcome::EmergencyLiquidation,
        &BlockOutcome::Freeze,
        false,
        true,
    );

    assert!(err.is_err());
    assert_eq!(
        err.unwrap_err(),
        "Contradiction: Cannot increase exposure when trading is frozen."
    );
}

#[test]
fn test_high_heat_recommendations() {
    let increase = IncreaseExposureEngine::evaluate(95, 80, 80, false, false);
    let reduce = ReduceExposureEngine::evaluate(95, 80, false, false);
    let close = CloseExposureEngine::evaluate(95, 80, false, false);
    let block = BlockEngine::evaluate(95, 80, false, false);

    assert_eq!(increase.outcome, IncreaseOutcome::Reject);
    assert_eq!(reduce.outcome, ReduceOutcome::ReduceAggressively);
    assert_eq!(close.outcome, CloseOutcome::EmergencyLiquidation);
    assert_eq!(block.outcome, BlockOutcome::Block);
}
