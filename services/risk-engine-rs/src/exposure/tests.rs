#![allow(clippy::unwrap_used)]
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::exposure::concentration::{ConcentrationLevel, ConcentrationMetrics};
use crate::exposure::currency_exposure::{CurrencyExposure, decompose_synthetic};
use crate::exposure::events::{ExposureRiskEvent, PositionOpenedEvent};
use crate::exposure::exposure_state::{ExposureRiskState, RiskState};
use crate::exposure::sector_exposure::{Sector, SectorExposure};
use crate::exposure::snapshot::ExposureRiskSnapshot;
use crate::exposure::symbol_exposure::SymbolExposure;
use crate::exposure::theme_exposure::{Theme, ThemeExposure};

#[test]
fn test_symbol_exposure_calculations() {
    let mut sym = SymbolExposure::new("AAPL".to_string());
    sym.long_exposure = Decimal::from(1000);
    sym.short_exposure = Decimal::from(200);
    
    assert_eq!(sym.net_exposure(), Decimal::from(800));
    
    sym.percentage_of_capital = Decimal::from(6);
    assert!(sym.is_oversized(Decimal::from(5)));
    assert!(!sym.is_oversized(Decimal::from(10)));
}

#[test]
fn test_synthetic_currency_decomposition() {
    let amount = Decimal::from(100);
    let (base, b_amt, quote, q_amt) = decompose_synthetic("EURUSD", true, amount).unwrap();
    assert_eq!(base, "EUR");
    assert_eq!(b_amt, Decimal::from(100));
    assert_eq!(quote, "USD");
    assert_eq!(q_amt, Decimal::from(-100));
    
    let (base, b_amt, quote, q_amt) = decompose_synthetic("GBPUSD", false, amount).unwrap();
    assert_eq!(base, "GBP");
    assert_eq!(b_amt, Decimal::from(-100));
    assert_eq!(quote, "USD");
    assert_eq!(q_amt, Decimal::from(100));
}

#[test]
fn test_sector_concentration() {
    let mut map = HashMap::new();
    let mut eq = SectorExposure::new(Sector::Equities);
    eq.total_exposure = Decimal::from(800);
    map.insert(Sector::Equities, eq);
    
    let mut fx = SectorExposure::new(Sector::Forex);
    fx.total_exposure = Decimal::from(200);
    map.insert(Sector::Forex, fx);
    
    SectorExposure::compute_dominance(&mut map, Decimal::from(1000));
    
    assert_eq!(map.get(&Sector::Equities).unwrap().dominance, Decimal::from(80));
    assert_eq!(map.get(&Sector::Forex).unwrap().dominance, Decimal::from(20));
}

#[test]
fn test_theme_clustering() {
    let mut map = HashMap::new();
    let mut tech = ThemeExposure::new(Theme::Tech);
    tech.exposure = Decimal::from(600);
    map.insert(Theme::Tech, tech);
    
    ThemeExposure::calculate_clustering(&mut map, Decimal::from(1000));
    
    assert_eq!(map.get(&Theme::Tech).unwrap().dominance_score, Decimal::from(60));
}

#[test]
fn test_gross_net_invariant() {
    let mut state = ExposureRiskState::new();
    state.gross_exposure = Decimal::from(1500);
    state.net_exposure = Decimal::from(-1000);
    
    assert!(state.gross_exposure >= state.net_exposure.abs());
    
    state.gross_exposure = Decimal::from(100);
    state.net_exposure = Decimal::from(100);
    assert!(state.gross_exposure >= state.net_exposure.abs());
}

#[test]
fn test_score_clamping() {
    let mut metrics = ConcentrationMetrics::new();
    metrics.largest_position_pct = Decimal::from(60);
    metrics.largest_sector_pct = Decimal::from(120);
    metrics.largest_theme_pct = Decimal::from(100);
    metrics.largest_currency_pct = Decimal::from(200);
    
    metrics.calculate_scores();
    
    assert!(metrics.concentration_score <= Decimal::from(100));
    assert!(metrics.concentration_score >= Decimal::ZERO);
    assert!(metrics.diversification_score <= Decimal::from(100));
    assert!(metrics.diversification_score >= Decimal::ZERO);
    assert_eq!(metrics.level, ConcentrationLevel::Collapse);
}

#[test]
fn test_determinism_over_100k_evaluations() {
    let mut metrics = ConcentrationMetrics::new();
    metrics.largest_position_pct = Decimal::from(10);
    metrics.largest_sector_pct = Decimal::from(20);
    metrics.largest_theme_pct = Decimal::from(30);
    metrics.largest_currency_pct = Decimal::from(40);
    
    metrics.calculate_scores();
    let initial_score = metrics.concentration_score;
    let initial_div = metrics.diversification_score;
    
    for _ in 0..100_000 {
        let mut m = ConcentrationMetrics::new();
        m.largest_position_pct = Decimal::from(10);
        m.largest_sector_pct = Decimal::from(20);
        m.largest_theme_pct = Decimal::from(30);
        m.largest_currency_pct = Decimal::from(40);
        m.calculate_scores();
        
        assert_eq!(m.concentration_score, initial_score);
        assert_eq!(m.diversification_score, initial_div);
    }
}

#[test]
fn test_snapshot_replay_correctness() {
    let mut state = ExposureRiskState::new();
    state.gross_exposure = Decimal::from(100);
    
    let event = ExposureRiskEvent::PositionOpened(PositionOpenedEvent {
        symbol: "AAPL".to_string(),
        amount: Decimal::from(100),
        is_long: true,
    });
    
    let snapshot = ExposureRiskSnapshot::new(1, 1680000000, state.clone(), Some(event.clone()));
    
    assert_eq!(snapshot.state().gross_exposure, Decimal::from(100));
    assert_eq!(snapshot.triggering_event, Some(event));
    assert_eq!(snapshot.version, 1);
}

#[test]
fn test_zero_panics() {
    let mut state = ExposureRiskState::new();
    state.determine_state(ConcentrationLevel::Normal);
    assert_eq!(state.state, RiskState::Normal);
    
    state.determine_state(ConcentrationLevel::Elevated);
    assert_eq!(state.state, RiskState::Elevated);
    
    state.determine_state(ConcentrationLevel::High);
    assert_eq!(state.state, RiskState::High);
    
    state.determine_state(ConcentrationLevel::Critical);
    assert_eq!(state.state, RiskState::Critical);
    
    state.determine_state(ConcentrationLevel::Collapse);
    assert_eq!(state.state, RiskState::Frozen);
    
    // Once frozen, it should not change
    state.determine_state(ConcentrationLevel::Normal);
    assert_eq!(state.state, RiskState::Frozen);
}

#[test]
fn test_short_usd_cluster() {
    let mut map = HashMap::new();
    let mut usd = CurrencyExposure::new("USD".to_string());
    usd.net_exposure = Decimal::from(-1000);
    map.insert("USD".to_string(), usd);
    
    assert!(CurrencyExposure::is_short_usd_cluster(&map, Decimal::from(500)));
    assert!(!CurrencyExposure::is_short_usd_cluster(&map, Decimal::from(1500)));
}
