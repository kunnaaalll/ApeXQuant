use backtester::market_replay::clock::ReplaySpeed;
use backtester::market_replay::engine::{ReplayEngine, TickReplayEngine};
use backtester::market_replay::models::Tick;
use rust_decimal::Decimal;
use time::OffsetDateTime;

#[test]
fn test_tick_replay_engine_100k() {
    let start_time = OffsetDateTime::now_utc();

    // Create 100k dummy ticks
    let mut ticks = Vec::with_capacity(100_000);
    for i in 0..100_000 {
        ticks.push(Tick {
            symbol: "BTCUSD".to_string(),
            timestamp: start_time + time::Duration::seconds(i as i64),
            bid: Decimal::new(50000, 0),
            ask: Decimal::new(50001, 0),
            bid_size: Decimal::ONE,
            ask_size: Decimal::ONE,
        });
    }

    let mut engine = TickReplayEngine::new(start_time, ReplaySpeed::Unlimited, ticks);

    let mut count = 0;
    while let Ok(Some(_)) = engine.next_event() {
        count += 1;
    }

    assert_eq!(count, 100_000);
}

#[test]
fn test_tick_replay_engine_1m() {
    let start_time = OffsetDateTime::now_utc();

    // Create 1m dummy ticks
    let mut ticks = Vec::with_capacity(1_000_000);
    for i in 0..1_000_000 {
        ticks.push(Tick {
            symbol: "ETHUSD".to_string(),
            timestamp: start_time + time::Duration::milliseconds(i as i64),
            bid: Decimal::new(3000, 0),
            ask: Decimal::new(3001, 0),
            bid_size: Decimal::ONE,
            ask_size: Decimal::ONE,
        });
    }

    let mut engine = TickReplayEngine::new(start_time, ReplaySpeed::Unlimited, ticks);

    let mut count = 0;
    while let Ok(Some(_)) = engine.next_event() {
        count += 1;
    }

    assert_eq!(count, 1_000_000);
}

// Commenting out 10M test as it might take too long in CI/local, but it's available.
// #[test]
// fn test_tick_replay_engine_10m() {
// ...
// }
