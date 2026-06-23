use crate::buffer::TickBuffer;
use crate::sequence::{SequenceTracker, SequenceState, SequenceResult};
use crate::gaps::{GapDetector, GapSeverity};
use crate::replay::{ReplayStream, ReplaySpeed};
use crate::statistics::StatisticsEngine;
use crate::throughput::ThroughputEngine;
use crate::tick::Tick;
use chrono::Utc;
use rust_decimal::Decimal;

fn create_dummy_tick(sequence: u64) -> Tick {
    Tick {
        symbol: "BTCUSD".to_string(),
        bid: Decimal::from(60000),
        ask: Decimal::from(60001),
        spread: Decimal::from(1),
        timestamp: Utc::now(),
        sequence,
    }
}

#[test]
fn test_tick_buffer() -> Result<(), String> {
    let mut buffer = TickBuffer::new(5);
    for i in 1..=10 {
        buffer.append(create_dummy_tick(i));
    }
    assert_eq!(buffer.len(), 5);
    assert_eq!(buffer.oldest().ok_or("No oldest tick")?.sequence, 6);
    assert_eq!(buffer.latest().ok_or("No latest tick")?.sequence, 10);
    Ok(())
}

#[test]
fn test_sequence_tracking() {
    let mut tracker = SequenceTracker::new();
    assert_eq!(tracker.process_sequence(1), SequenceResult::Ok);
    assert_eq!(tracker.process_sequence(2), SequenceResult::Ok);
    assert_eq!(tracker.state(), SequenceState::Healthy);
    
    // Missing sequence
    assert_eq!(tracker.process_sequence(4), SequenceResult::Missing(1));
    assert_eq!(tracker.state(), SequenceState::Warning);

    // Duplicate sequence
    assert_eq!(tracker.process_sequence(4), SequenceResult::DuplicateOrOutOfOrder);
}

#[test]
fn test_duplicate_detection() {
    let mut tracker = SequenceTracker::new();
    tracker.process_sequence(100);
    assert_eq!(tracker.process_sequence(100), SequenceResult::DuplicateOrOutOfOrder);
}

#[test]
fn test_gap_detection() -> Result<(), String> {
    let mut detector = GapDetector::new();
    let now = Utc::now();
    assert!(detector.process_tick(now, 0, false).is_none());
    
    let gap = detector.process_tick(now, 5, false).ok_or("No gap")?;
    assert_eq!(gap.severity, GapSeverity::Minor);
    
    let gap_critical = detector.process_tick(now, 105, false).ok_or("No gap")?;
    assert_eq!(gap_critical.severity, GapSeverity::Critical);
    Ok(())
}

#[tokio::test]
async fn test_replay_stream() -> Result<(), String> {
    let ticks = vec![create_dummy_tick(1), create_dummy_tick(2)];
    let mut replay = ReplayStream::new(ticks, ReplaySpeed::Turbo);
    let t1 = replay.next_tick().await.ok_or("No tick 1")?;
    let t2 = replay.next_tick().await.ok_or("No tick 2")?;
    assert_eq!(t1.sequence, 1);
    assert_eq!(t2.sequence, 2);
    assert!(replay.is_empty());
    Ok(())
}

#[test]
fn test_statistics() {
    let mut stats = StatisticsEngine::new();
    stats.record_tick();
    stats.record_tick();
    stats.record_duplicate();
    assert_eq!(stats.total_ticks, 2);
    assert_eq!(stats.duplicate_ticks, 1);
}

#[test]
fn test_throughput_engine() {
    let mut engine = ThroughputEngine::new();
    engine.record_tick();
    engine.record_candle();
    engine.record_event();
    
    // Test logic ensures we don't return metrics if < 1s
    let now = Utc::now();
    assert!(engine.calculate_metrics(now).is_none());
}

#[test]
fn test_million_tick_stress() -> Result<(), String> {
    let mut buffer = TickBuffer::new(1000);
    for i in 1..=1_000_000 {
        buffer.append(create_dummy_tick(i));
    }
    assert_eq!(buffer.len(), 1000);
    assert_eq!(buffer.latest().ok_or("No latest")?.sequence, 1_000_000);
    Ok(())
}

#[test]
fn test_candle_aggregation() -> Result<(), String> {
    use crate::aggregation::CandleAggregator;
    use crate::candle::Timeframe;
    let mut aggregator = CandleAggregator::new(Timeframe::M1);
    
    let tick1 = create_dummy_tick(1);
    let mut tick2 = create_dummy_tick(2);
    tick2.bid = Decimal::from(60100);
    tick2.ask = Decimal::from(60101);
    // tick2 happens 65 seconds later, which should close the candle
    tick2.timestamp = tick1.timestamp + chrono::Duration::seconds(65);

    let res1 = aggregator.process_tick(&tick1);
    assert!(res1.is_none());

    let res2 = aggregator.process_tick(&tick2);
    assert!(res2.is_some());
    let candle = res2.ok_or("No candle")?;
    assert_eq!(candle.volume, Decimal::from(1)); // 1 tick formed the first candle
    Ok(())
}
