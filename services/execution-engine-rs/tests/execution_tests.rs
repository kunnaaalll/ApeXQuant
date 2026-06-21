use execution_engine::execution::smart::{ExecutionScore, Priority, RoutingDecision, RoutingState, Urgency};
use execution_engine::fills::{AveragePriceCalculator, FillState, PartialFillEngine};
use execution_engine::liquidity::{AvailabilityScore, DepthScore, LiquidityRegime, OrderBookImbalance, SpreadQuality};
use execution_engine::order_split::{IcebergSplitter, TwapSplitter, VwapSplitter};
use execution_engine::policies::fok::FokPolicy;
use execution_engine::policies::gtc::GtcPolicy;
use execution_engine::policies::ioc::IocPolicy;
use execution_engine::policies::PolicyState;
use execution_engine::slippage::SlippageScore;
use execution_engine::events::SmartExecutionEvent;
use execution_engine::snapshots::SmartExecutionSnapshot;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

#[test]
fn test_partial_fill() {
    let mut engine = PartialFillEngine::new(dec!(100));
    assert_eq!(engine.state(), FillState::None);
    assert_eq!(engine.remaining_quantity(), dec!(100));

    let _ = engine.add_fill(dec!(30)).unwrap();
    assert_eq!(engine.state(), FillState::Partial);
    assert_eq!(engine.remaining_quantity(), dec!(70));

    let _ = engine.add_fill(dec!(70)).unwrap();
    assert_eq!(engine.state(), FillState::Completed);
    assert_eq!(engine.remaining_quantity(), dec!(0));
}

#[test]
fn test_average_price() {
    let mut calc = AveragePriceCalculator::new();
    calc.add_fill(dec!(100), dec!(10)); // value 1000
    calc.add_fill(dec!(110), dec!(10)); // value 1100
    // total value 2100, total qty 20 -> avg 105
    assert_eq!(calc.average_price(), Some(dec!(105)));
}

#[test]
fn test_twap_split() {
    let parent = Uuid::new_v4();
    let orders = TwapSplitter::split(parent, dec!(100), 3);
    assert_eq!(orders.len(), 3);
    assert_eq!(orders[0].quantity, dec!(33.3333));
    assert_eq!(orders[1].quantity, dec!(33.3333));
    assert_eq!(orders[2].quantity, dec!(33.3334));
}

#[test]
fn test_vwap_split() {
    let parent = Uuid::new_v4();
    let weights = vec![dec!(10), dec!(20), dec!(70)]; // Total 100
    let orders = VwapSplitter::split(parent, dec!(1000), &weights);
    assert_eq!(orders.len(), 3);
    assert_eq!(orders[0].quantity, dec!(100));
    assert_eq!(orders[1].quantity, dec!(200));
    assert_eq!(orders[2].quantity, dec!(700));
}

#[test]
fn test_iceberg_visibility() {
    let parent = Uuid::new_v4();
    let orders = IcebergSplitter::split(parent, dec!(250), dec!(100));
    assert_eq!(orders.len(), 3);
    assert_eq!(orders[0].quantity, dec!(100));
    assert_eq!(orders[1].quantity, dec!(100));
    assert_eq!(orders[2].quantity, dec!(50));
}

#[test]
fn test_slippage_bounds() {
    let max_slip = dec!(10);
    // perfect fill
    let score = SlippageScore::calculate(dec!(-1), max_slip);
    assert_eq!(score, dec!(100));
    // worst fill
    let score = SlippageScore::calculate(dec!(15), max_slip);
    assert_eq!(score, dec!(0));
    // half slip
    let score = SlippageScore::calculate(dec!(5), max_slip);
    assert_eq!(score, dec!(50));
}

#[test]
fn test_execution_score_bounds() {
    // Check elite execution
    let score = ExecutionScore::new(dec!(100), dec!(100), dec!(100), dec!(100)).unwrap();
    assert_eq!(score.final_score, dec!(100));
    assert!(matches!(score.grade(), execution_engine::execution::smart::ExecutionGrade::Elite));

    // Check bounds validation
    assert!(ExecutionScore::new(dec!(101), dec!(100), dec!(100), dec!(100)).is_err());
    assert!(ExecutionScore::new(dec!(-1), dec!(100), dec!(100), dec!(100)).is_err());
}

#[test]
fn test_liquidity_regimes() {
    // Tests for our liquidity tools (depth, availability, spread_quality, imbalance)
    assert_eq!(DepthScore::calculate(dec!(50), dec!(100)), dec!(50));
    assert_eq!(DepthScore::calculate(dec!(150), dec!(100)), dec!(100)); // Capped at 100

    assert_eq!(AvailabilityScore::calculate(dec!(0.999)), dec!(99.90));
    
    assert_eq!(SpreadQuality::calculate(dec!(1), dec!(2)), dec!(100)); // half the historical spread is perfect
    assert_eq!(SpreadQuality::calculate(dec!(3), dec!(2)), dec!(50));  // 1.5x historical
    assert_eq!(SpreadQuality::calculate(dec!(4), dec!(2)), dec!(0));   // 2x historical is zero
    
    assert_eq!(OrderBookImbalance::calculate(dec!(70), dec!(30)), dec!(0.4000));
}

#[test]
fn test_policy_transitions() {
    // IOC test
    let mut ioc = IocPolicy::new();
    assert!(ioc.transition(PolicyState::Active).is_ok());
    assert!(ioc.transition(PolicyState::PartiallyFilled).is_ok());
    assert!(ioc.transition(PolicyState::Cancelled).is_ok()); // Cancel remainder

    // FOK test
    let mut fok = FokPolicy::new();
    assert!(fok.transition(PolicyState::Active).is_ok());
    assert!(fok.transition(PolicyState::PartiallyFilled).is_err()); // Not allowed for FOK
    assert!(fok.transition(PolicyState::Filled).is_ok());

    // GTC test
    let mut gtc = GtcPolicy::new();
    assert!(gtc.transition(PolicyState::Active).is_ok());
    assert!(gtc.transition(PolicyState::PartiallyFilled).is_ok());
    assert!(gtc.transition(PolicyState::Filled).is_ok());
}

#[test]
fn test_event_replay() {
    let id = Uuid::new_v4();
    let mut snapshot = SmartExecutionSnapshot::new(id, dec!(100));

    let events = vec![
        SmartExecutionEvent::RoutingDecisionMade {
            order_id: id,
            urgency: Urgency::Balanced,
            priority: Priority::High,
            decision: RoutingDecision::new("BINANCE", RoutingState::Primary),
        },
        SmartExecutionEvent::PartialFillRecorded {
            order_id: id,
            filled_qty: dec!(40),
            remaining_qty: dec!(60),
            state: FillState::Partial,
        },
        SmartExecutionEvent::LiquidityEvaluated {
            order_id: id,
            regime: LiquidityRegime::Healthy,
        },
    ];

    for event in events {
        match event {
            SmartExecutionEvent::RoutingDecisionMade { urgency, priority, decision, .. } => {
                snapshot.urgency = urgency;
                snapshot.priority = priority;
                snapshot.current_routing = Some(decision);
            }
            SmartExecutionEvent::PartialFillRecorded { filled_qty, remaining_qty, state, .. } => {
                snapshot.filled_qty = filled_qty;
                snapshot.remaining_qty = remaining_qty;
                snapshot.fill_state = state;
            }
            SmartExecutionEvent::LiquidityEvaluated { regime, .. } => {
                snapshot.current_liquidity_regime = regime;
            }
            _ => {}
        }
    }

    assert_eq!(snapshot.filled_qty, dec!(40));
    assert_eq!(snapshot.remaining_qty, dec!(60));
    assert_eq!(snapshot.fill_state, FillState::Partial);
    assert_eq!(snapshot.current_liquidity_regime, LiquidityRegime::Healthy);
    assert_eq!(snapshot.urgency, Urgency::Balanced);
    assert_eq!(snapshot.priority, Priority::High);
}

#[test]
fn test_determinism_100k_iterations() {
    // 100,000 loops. Zero drift validation using only Decimal.
    let mut current_price = dec!(100.0);
    let mut total_filled = dec!(0);
    
    for _ in 0..100_000 {
        // Deterministic updates with exact math
        current_price = (current_price * dec!(1.00001)).trunc_with_scale(4);
        total_filled += dec!(0.001);
    }
    
    // Check exact matches
    assert_eq!(total_filled, dec!(100));
    assert_eq!(current_price, dec!(263.3815));
}
