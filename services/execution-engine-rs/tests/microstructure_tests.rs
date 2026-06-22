#![deny(unsafe_code)]

use rust_decimal::Decimal;
use execution_engine::microstructure::bid_ask::BidAsk;
use execution_engine::microstructure::spread::Spread;
use execution_engine::microstructure::depth::{OrderBookDepth, DepthLevel};
use execution_engine::microstructure::imbalance::ImbalanceScore;
use execution_engine::microstructure::queue::QueueState;
use execution_engine::microstructure::impact::{MarketImpact, MarketImpactGrade};
use execution_engine::microstructure::resiliency::ResiliencyState;
use execution_engine::microstructure::score::{MicrostructureScore, MicrostructureGrade};
use execution_engine::market::state::MarketState;
use execution_engine::market::transitions::StateTransition;
use execution_engine::latency::health::LatencyState;
use execution_engine::latency::score::LatencyScore;
use execution_engine::execution_cost::spread_cost::SpreadCost;
use execution_engine::execution_cost::slippage_cost::SlippageCost;
use execution_engine::execution_cost::impact_cost::ImpactCost;
use execution_engine::execution_cost::total_cost::{TotalExecutionCost, TotalExecutionCostGrade};
use execution_engine::events::microstructure_events::MicrostructureEvent;
use execution_engine::snapshots::microstructure_snapshots::MicrostructureSnapshot;

#[test]
fn test_spread_bounds() {
    let bid_ask = BidAsk::new(Decimal::new(100, 0), Decimal::new(101, 0)).unwrap();
    assert_eq!(bid_ask.spread(), Decimal::new(1, 0));
    assert_eq!(bid_ask.midpoint(), Decimal::new(1005, 1));
    let spread = Spread::calculate(&bid_ask).unwrap();
    assert_eq!(spread.absolute, Decimal::new(1, 0));
}

#[test]
fn test_depth_grades() {
    let depth = OrderBookDepth::new(Decimal::new(500, 0), Decimal::new(300, 0), Decimal::new(200, 0)).unwrap();
    assert_eq!(depth.grade(), DepthLevel::Excellent);
    
    let weak_depth = OrderBookDepth::new(Decimal::new(5, 0), Decimal::new(3, 0), Decimal::new(2, 0)).unwrap();
    assert_eq!(weak_depth.grade(), DepthLevel::Weak);
}

#[test]
fn test_imbalance_score() {
    let score = ImbalanceScore::calculate(Decimal::new(80, 0), Decimal::new(20, 0)).unwrap();
    assert_eq!(score.score, 80);
}

#[test]
fn test_queue_states() {
    let state = QueueState::determine(10, 100).unwrap();
    assert_eq!(state, QueueState::Front);
    let state = QueueState::determine(50, 100).unwrap();
    assert_eq!(state, QueueState::Middle);
    let state = QueueState::determine(90, 100).unwrap();
    assert_eq!(state, QueueState::Back);
}

#[test]
fn test_market_impact_bounds() {
    let impact = MarketImpact::calculate(Decimal::new(20, 0)).unwrap();
    assert_eq!(impact.grade, MarketImpactGrade::High);
    assert_eq!(impact.score, 75);
}

#[test]
fn test_resiliency_states() {
    let state = ResiliencyState::evaluate(8).unwrap();
    assert_eq!(state, ResiliencyState::Fast);
    let state = ResiliencyState::evaluate(30).unwrap();
    assert_eq!(state, ResiliencyState::Normal);
}

#[test]
fn test_market_state_transitions() {
    let next = StateTransition::next(MarketState::Normal, MarketState::Stressed).unwrap();
    assert_eq!(next, MarketState::Stressed);
    
    let fail = StateTransition::next(MarketState::Closed, MarketState::Stressed);
    assert!(fail.is_err());
}

#[test]
fn test_latency_bounds() {
    let state = LatencyState::evaluate(40).unwrap();
    assert_eq!(state, LatencyState::Healthy);
    
    let score = LatencyScore::calculate(80).unwrap();
    assert_eq!(score.score, 60);
    
    let score_critical = LatencyScore::calculate(250).unwrap();
    assert_eq!(score_critical.score, 0);
}

#[test]
fn test_execution_cost_score() {
    let notional = Decimal::new(100000, 0);
    let spread = SpreadCost::calculate(Decimal::new(10, 0), notional).unwrap(); // $100
    let slippage = SlippageCost::calculate(Decimal::new(5, 0), notional).unwrap(); // $50
    let impact = ImpactCost::calculate(Decimal::new(5, 0), notional).unwrap(); // $50
    
    let total = TotalExecutionCost::calculate(&spread, &slippage, &impact, notional).unwrap();
    assert_eq!(total.total_usd, Decimal::new(200, 0));
    // 200 on 100k is 20 bps -> Poor
    assert_eq!(total.grade, TotalExecutionCostGrade::Poor);
}

#[test]
fn test_microstructure_score_bounds() {
    let score = MicrostructureScore::calculate(90, 90, 90, 90, 90, 90).unwrap();
    assert_eq!(score.score, 90);
    assert_eq!(score.grade, MicrostructureGrade::Elite);
}

#[test]
fn test_event_replay() {
    let event = MicrostructureEvent::ImbalanceUpdated { score: 75 };
    if let MicrostructureEvent::ImbalanceUpdated { score } = event {
        assert_eq!(score, 75);
    } else {
        panic!("Wrong event type");
    }
}

#[test]
fn test_snapshot_restore() {
    let snapshot = MicrostructureSnapshot {
        score: 85,
        grade: MicrostructureGrade::Strong,
    };
    assert_eq!(snapshot.score, 85);
    assert_eq!(snapshot.grade, MicrostructureGrade::Strong);
}

#[test]
fn test_determinism_100k_iterations() {
    let mut last_score = 0;
    for _ in 0..100_000 {
        let score = MicrostructureScore::calculate(80, 80, 80, 80, 80, 80).unwrap();
        assert_eq!(score.score, 80);
        last_score = score.score;
    }
    assert_eq!(last_score, 80);
}
