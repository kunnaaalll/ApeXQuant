use crate::{RiskEngine, RiskInputs, MarketSession};
use rust_decimal::Decimal;

pub struct DeterminismResult {
    pub iterations: usize,
    pub determinism_pct: f64,
    pub mismatched_fields: Vec<String>,
}

pub async fn run_determinism_validation(engine: &RiskEngine) -> DeterminismResult {
    let mut inputs = RiskInputs {
        equity: Decimal::new(100000, 0),
        balance: Decimal::new(100000, 0),
        symbol: "BTCUSD".to_string(),
        direction: 1,
        entry_price: Decimal::new(65000, 0),
        stop_loss: Decimal::new(64000, 0),
        take_profit: Some(Decimal::new(68000, 0)),
        signal_confidence: Decimal::from_str_exact("0.85").unwrap(),
        confluence_score: Decimal::from_str_exact("8.0").unwrap(),
        regime_quality: Decimal::from_str_exact("0.9").unwrap(),
        pattern_quality: Decimal::from_str_exact("0.8").unwrap(),
        atr: Some(Decimal::new(1000, 0)),
        spread: Decimal::new(10, 0),
        open_positions: Vec::new(),
        daily_pnl: Decimal::ZERO,
        daily_trades: 0,
        recent_trades: Vec::new(),
        session: MarketSession::NewYork,
    };

    let first_result = engine.assess(&inputs).await.unwrap();
    let mut is_deterministic = true;
    let iterations = 100_000;

    for _ in 0..iterations {
        let result = engine.assess(&inputs).await.unwrap();
        // The latency and timestamp will differ, but all core risk logic should be identical
        if result.approved != first_result.approved ||
           result.lot_size != first_result.lot_size ||
           result.risk_percent != first_result.risk_percent ||
           result.risk_profile != first_result.risk_profile {
            is_deterministic = false;
            break;
        }
    }

    DeterminismResult {
        iterations,
        determinism_pct: if is_deterministic { 100.0 } else { 0.0 },
        mismatched_fields: vec![],
    }
}
