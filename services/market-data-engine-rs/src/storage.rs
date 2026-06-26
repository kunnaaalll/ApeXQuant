use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use crate::event_bus::EventBusPublisher;
use apex_protos::events::{Event, event::Payload, TickReceivedEvent};
use apex_protos::common::{Price, Timestamp, Uuid};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickRecord {
    pub symbol:    String,
    pub sequence:  i64,
    pub bid:       Decimal,
    pub ask:       Decimal,
    pub spread:    Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleRecord {
    pub symbol:     String,
    pub timeframe:  String,
    pub open:       Decimal,
    pub high:       Decimal,
    pub low:        Decimal,
    pub close:      Decimal,
    pub volume:     Decimal,
    pub start_time: DateTime<Utc>,
    pub end_time:   DateTime<Utc>,
}

pub struct TickRepository {
    pool: Option<Pool<Postgres>>,
    event_bus: Option<Arc<EventBusPublisher>>,
}

impl TickRepository {
    pub fn new(pool: Option<Pool<Postgres>>, event_bus: Option<Arc<EventBusPublisher>>) -> Self { Self { pool, event_bus } }

    pub async fn save_tick(&self, record: &TickRecord) -> Result<(), sqlx::Error> { 
        if let Some(bus) = &self.event_bus {
            let tick_event = TickReceivedEvent {
                symbol: record.symbol.clone(),
                timestamp: None,
                bid: Some(Price { value: record.bid.to_string(), digits: 0, currency: "USD".to_string() }),
                ask: Some(Price { value: record.ask.to_string(), digits: 0, currency: "USD".to_string() }),
                bid_provider: "binance".to_string(),
                ask_provider: "binance".to_string(),
            };

            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            let event = Event {
                event_id: Some(Uuid { value: uuid::Uuid::new_v4().as_bytes().to_vec() }),
                spec_version: None,
                occurred_at: Some(Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
                published_at: Some(Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 }),
                event_type: "TickReceivedEvent".to_string(),
                source_service: "market-data-engine".to_string(),
                topic: "market_data.ticks".to_string(),
                correlation: None,
                causation_id: "".to_string(),
                deduplication_key: "".to_string(),
                payload: Some(Payload::TickReceived(tick_event)),
                payload_hash: vec![],
            };

            if let Err(e) = bus.publish(event).await {
                tracing::warn!("Failed to publish TickReceivedEvent: {}", e);
            }
        }
        Ok(()) 
    }

    pub async fn load_ticks_ordered(&self, _symbol: &str, _from_sequence: i64, _limit: i64) -> Result<Vec<TickRecord>, sqlx::Error> { Ok(vec![]) }

    pub async fn count_ticks(&self, _symbol: &str) -> Result<i64, sqlx::Error> { Ok(0) }
}

pub struct CandleRepository {
    pool: Option<Pool<Postgres>>,
}

impl CandleRepository {
    pub fn new(pool: Option<Pool<Postgres>>) -> Self { Self { pool } }

    pub async fn save_candle(&self, _record: &CandleRecord) -> Result<(), sqlx::Error> { Ok(()) }

    pub async fn load_candles_ordered(&self, _symbol: &str, _timeframe: &str, _from: DateTime<Utc>, _to: DateTime<Utc>) -> Result<Vec<CandleRecord>, sqlx::Error> { Ok(vec![]) }
}

pub struct MarketDataStore {
    pub pool: Option<Pool<Postgres>>,
    pub ticks: TickRepository,
    pub candles: CandleRepository,
    pub event_bus: Option<Arc<EventBusPublisher>>,
}

impl MarketDataStore {
    pub fn new(pool: Pool<Postgres>, event_bus: Option<Arc<EventBusPublisher>>) -> Self {
        Self {
            pool: Some(pool.clone()),
            ticks: TickRepository::new(Some(pool.clone()), event_bus.clone()),
            candles: CandleRepository::new(Some(pool)),
            event_bus,
        }
    }
}
