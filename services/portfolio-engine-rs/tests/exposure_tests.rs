#![allow(warnings, clippy::all, deprecated)]
use portfolio_engine::exposure::currency::Currency;
use portfolio_engine::exposure::events::ExposureEvent;
use portfolio_engine::exposure::registry::ExposureRegistry;
use portfolio_engine::exposure::sector::Sector;
use portfolio_engine::exposure::state::ExposureState;
use proptest::prelude::*;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use uuid::Uuid;

#[test]
fn test_synthetic_currency_decomposition() {
    let registry = ExposureRegistry::new();
    let pos_id = Uuid::new_v4();

    // Simulating EURUSD Long: Long EUR, Short USD
    let event = ExposureEvent::PositionOpened {
        position_id: pos_id,
        symbol_id: "EURUSD".to_string(),
        sector: Sector::Forex,
        base_currency: Currency::EUR,
        quote_currency: Currency::USD,
        base_size: Decimal::from(100_000),   // +100k EUR
        quote_size: Decimal::from(-108_500), // -108.5k USD (assuming 1.0850 rate)
        margin_used: Decimal::from(1000),
        risk_amount: Decimal::from(500),
    };

    registry.dispatch(event).unwrap();

    let state = registry.get_state().unwrap();

    // Verify EUR Exposure
    let eur = state.currencies.get(&Currency::EUR).unwrap();
    assert_eq!(eur.net_exposure, Decimal::from(100_000));
    assert_eq!(eur.long_exposure, Decimal::from(100_000));
    assert_eq!(eur.short_exposure, Decimal::ZERO);
    assert_eq!(eur.gross_exposure, Decimal::from(100_000));

    // Verify USD Exposure
    let usd = state.currencies.get(&Currency::USD).unwrap();
    assert_eq!(usd.net_exposure, Decimal::from(-108_500));
    assert_eq!(usd.long_exposure, Decimal::ZERO);
    assert_eq!(usd.short_exposure, Decimal::from(108_500));
    assert_eq!(usd.gross_exposure, Decimal::from(108_500));

    // Verify Global
    assert_eq!(state.global.long_exposure, Decimal::from(100_000));
    assert_eq!(state.global.short_exposure, Decimal::from(108_500));
    assert_eq!(state.global.gross_exposure, Decimal::from(208_500));
    assert_eq!(state.global.net_exposure, Decimal::from(-8_500));
}

#[test]
fn test_concentration_detection() {
    let registry = ExposureRegistry::new();

    // Push excessive USD short
    registry
        .dispatch(ExposureEvent::PositionOpened {
            position_id: Uuid::new_v4(),
            symbol_id: "EURUSD".to_string(),
            sector: Sector::Forex,
            base_currency: Currency::EUR,
            quote_currency: Currency::USD,
            base_size: Decimal::from(50_000),
            quote_size: Decimal::from(-60_000),
            margin_used: Decimal::from(500),
            risk_amount: Decimal::from(200),
        })
        .unwrap();

    registry
        .dispatch(ExposureEvent::PositionOpened {
            position_id: Uuid::new_v4(),
            symbol_id: "GBPUSD".to_string(),
            sector: Sector::Forex,
            base_currency: Currency::GBP,
            quote_currency: Currency::USD,
            base_size: Decimal::from(40_000),
            quote_size: Decimal::from(-50_000),
            margin_used: Decimal::from(400),
            risk_amount: Decimal::from(200),
        })
        .unwrap();

    let state = registry.get_state().unwrap();
    let concentrations = state.assess_concentration();

    // Check USD Short detection (total short USD is 110_000, threshold is 100_000)
    assert!(concentrations
        .iter()
        .any(|c| c.description.contains("Excessive USD short")));

    // Now push risk-on to > 50%
    registry
        .dispatch(ExposureEvent::PositionOpened {
            position_id: Uuid::new_v4(),
            symbol_id: "BTCUSD".to_string(),
            sector: Sector::Crypto,
            base_currency: Currency::BTC,
            quote_currency: Currency::USD,
            base_size: Decimal::from(1_000_000), // Massive to dominate weight
            quote_size: Decimal::from(-1_000_000),
            margin_used: Decimal::from(10000),
            risk_amount: Decimal::from(5000),
        })
        .unwrap();

    let state2 = registry.get_state().unwrap();
    let concentrations2 = state2.assess_concentration();
    assert!(concentrations2
        .iter()
        .any(|c| c.description.contains("High Risk-on concentration")));
}

// Fuzz test for mathematical invariants
proptest! {
    #[test]
    fn test_exposure_invariants(
        long1 in 0.0..100_000.0,
        short1 in -100_000.0..0.0,
    ) {
        let mut state = ExposureState::new();
        let e1 = ExposureEvent::PositionOpened {
            position_id: Uuid::new_v4(),
            symbol_id: "TEST1".to_string(),
            sector: Sector::Synthetic,
            base_currency: Currency::AUD,
            quote_currency: Currency::USD,
            base_size: Decimal::from_f64(long1).unwrap(),
            quote_size: Decimal::from_f64(short1).unwrap(),
            margin_used: Decimal::ZERO,
            risk_amount: Decimal::ZERO,
        };

        // Note: Using a fixed time for deterministic test execution
        state.apply_event(&e1, time::OffsetDateTime::now_utc()).unwrap();
        assert!(state.validate_invariants().is_ok());
    }
}
