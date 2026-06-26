use portfolio_engine::portfolio::events::PortfolioEvent;
use portfolio_engine::portfolio::state::PortfolioState;
use proptest::prelude::*;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use uuid::Uuid;

#[test]
fn test_default_state_invariants() {
    let mut state = PortfolioState::new();
    assert!(state.validate_invariants().is_ok());
}

#[test]
fn test_apply_deposit() {
    let mut state = PortfolioState::new();
    let event = PortfolioEvent::Deposit {
        amount: Decimal::from(1000),
    };
    let timestamp = OffsetDateTime::now_utc();
    
    assert!(state.apply_event(&event, timestamp).is_ok());
    assert_eq!(state.balance, Decimal::from(1000));
    assert_eq!(state.equity, Decimal::from(1000));
    assert_eq!(state.free_margin, Decimal::from(1000));
    assert_eq!(state.used_margin, Decimal::from(0));
}

#[test]
fn test_position_opened_and_pnl() {
    let mut state = PortfolioState::new();
    let timestamp = OffsetDateTime::now_utc();

    // 1. Deposit
    state.apply_event(&PortfolioEvent::Deposit { amount: Decimal::from(10000) }, timestamp).unwrap();

    // 2. Open Position
    state.apply_event(&PortfolioEvent::PositionOpened {
        position_id: Uuid::new_v4(),
        margin_used: Decimal::from(1000),
        exposure: Decimal::from(10000),
    }, timestamp).unwrap();

    assert_eq!(state.balance, Decimal::from(10000));
    assert_eq!(state.equity, Decimal::from(10000));
    assert_eq!(state.used_margin, Decimal::from(1000));
    assert_eq!(state.free_margin, Decimal::from(9000)); // 10000 - 1000
    assert_eq!(state.margin_level, Decimal::from(10)); // 10000 / 1000 = 1000%
    assert_eq!(state.active_positions, 1);

    // 3. PnL Update (floating +500)
    state.apply_event(&PortfolioEvent::PnlUpdate {
        position_id: Uuid::new_v4(),
        pnl_delta: Decimal::from(500),
    }, timestamp).unwrap();

    assert_eq!(state.balance, Decimal::from(10000));
    assert_eq!(state.equity, Decimal::from(10500)); // 10000 + 500
    assert_eq!(state.free_margin, Decimal::from(9500)); // 10500 - 1000
    assert_eq!(state.margin_level, Decimal::from_f64(10.5).unwrap());

    // 4. Position Closed
    state.apply_event(&PortfolioEvent::PositionClosed {
        position_id: Uuid::new_v4(),
        realized_pnl: Decimal::from(500),
        margin_released: Decimal::from(1000),
        exposure_released: Decimal::from(10000),
    }, timestamp).unwrap();

    // After close, balance increases by realized PnL, floating PnL should be externally adjusted back,
    // but in this event stream, usually we have a PnlUpdate delta back to 0 before close, 
    // or the engine adjusts it.
    // Let's explicitly zero out the floating PnL for the position since it's closed.
    state.apply_event(&PortfolioEvent::PnlUpdate {
        position_id: Uuid::new_v4(),
        pnl_delta: Decimal::from(-500),
    }, timestamp).unwrap();

    assert_eq!(state.balance, Decimal::from(10500));
    assert_eq!(state.equity, Decimal::from(10500));
    assert_eq!(state.free_margin, Decimal::from(10500));
    assert_eq!(state.used_margin, Decimal::from(0));
    assert_eq!(state.active_positions, 0);
}

// Property tests for invariant enforcement
proptest! {
    #[test]
    fn test_equity_invariant(
        balance in proptest::num::f64::NORMAL,
        floating_pnl in proptest::num::f64::NORMAL,
    ) {
        let mut state = PortfolioState::new();
        state.balance = Decimal::from_f64(balance).unwrap_or(Decimal::ZERO);
        state.floating_pnl = Decimal::from_f64(floating_pnl).unwrap_or(Decimal::ZERO);
        // Force equity to be correct
        state.equity = state.balance + state.floating_pnl;
        state.free_margin = state.equity; // assuming 0 used_margin

        assert!(state.validate_invariants().is_ok());
    }

    #[test]
    fn test_margin_level_invariant(
        equity_val in 1000.0..10000.0,
        used_margin_val in 100.0..5000.0,
    ) {
        let mut state = PortfolioState::new();
        state.equity = Decimal::from_f64(equity_val).unwrap();
        state.balance = state.equity; // no floating pnl
        state.used_margin = Decimal::from_f64(used_margin_val).unwrap();
        state.free_margin = state.equity - state.used_margin;

        assert!(state.validate_invariants().is_ok());
        
        let expected_margin_level = state.equity / state.used_margin;
        assert_eq!(state.margin_level, expected_margin_level);
    }
}
