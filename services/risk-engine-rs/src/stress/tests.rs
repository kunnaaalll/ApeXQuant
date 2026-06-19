use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::correlation::CorrelationCollapseEngine;
use super::leverage::{LeverageCascadeEngine, LeverageState};
use super::liquidity::LiquidityState;
use super::scenarios::HistoricalScenario;
use super::severity::Severity;
use super::snapshot::StressSnapshot;
use super::survival::SurvivalEngine;
use super::volatility::{VolatilityShockEngine, VolatilityState};

#[test]
fn test_survival_score_bounds() {
    let score_max = SurvivalEngine::compute_score(
        VolatilityState::Normal,
        LiquidityState::Healthy,
        LeverageState::Stable,
        dec!(0.0),
    );
    assert_eq!(score_max, dec!(100.0));

    let score_min = SurvivalEngine::compute_score(
        VolatilityState::Collapse,
        LiquidityState::Frozen,
        LeverageState::Collapse,
        dec!(1.0),
    );
    assert!(score_min >= dec!(0.0));
    assert!(score_min <= dec!(100.0));
    assert_eq!(score_min, dec!(0.0));
}

#[test]
fn test_determinism_100k_iterations() {
    let scenario = HistoricalScenario::Lehman2008;

    let base_vol = dec!(0.20);
    let vol_engine = VolatilityShockEngine::new(base_vol);

    let mut previous_vol: Option<Decimal> = None;

    for _ in 0..100_000 {
        let shocked_vol = vol_engine.apply_shock(scenario.volatility_multiplier());
        
        if let Some(prev) = previous_vol {
            assert_eq!(shocked_vol, prev, "Determinism failed: value drifted");
        }
        previous_vol = Some(shocked_vol);
    }
}

#[test]
fn test_correlation_convergence() {
    let base_corr = dec!(0.2);
    let corr_engine = CorrelationCollapseEngine::new(base_corr);

    let scenario = HistoricalScenario::FlashCrash; // Mult: 1.5
    let collapsed_corr = corr_engine.apply_collapse(scenario.correlation_multiplier());

    assert!(collapsed_corr > base_corr, "Correlation should converge towards 1.0");
    assert!(collapsed_corr <= dec!(1.0));
}

#[test]
fn test_leverage_cascade() {
    let base_leverage = dec!(3.0);
    let lev_engine = LeverageCascadeEngine::new(base_leverage);

    let scenario = HistoricalScenario::CovidCrash2020; // Mult: 1.5
    let cascaded_leverage = lev_engine.apply_cascade(scenario.leverage_amplification());

    assert!(cascaded_leverage > base_leverage);
    assert_eq!(cascaded_leverage, dec!(4.5));
    
    let state = lev_engine.evaluate_state(cascaded_leverage);
    assert_eq!(state, LeverageState::Danger);
}

#[test]
fn test_event_rebuild() -> Result<(), serde_json::Error> {
    let snapshot = StressSnapshot {
        scenario: HistoricalScenario::DotComBubble,
        severity: Severity::High,
        survival_score: dec!(45.5),
        timestamp: 1625097600,
        version: 1,
    };

    let serialized = serde_json::to_string(&snapshot)?;
    let deserialized: StressSnapshot = serde_json::from_str(&serialized)?;

    assert_eq!(snapshot, deserialized);
    Ok(())
}
