use crate::event_bus::EventBusPublisher;
use apex_protos::common::{Price, Timestamp, Uuid};
use apex_protos::events::{event::Payload, Event, TickReceivedEvent};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickRecord {
    pub symbol: String,
    pub sequence: i64,
    pub bid: Decimal,
    pub ask: Decimal,
    pub spread: Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleRecord {
    pub symbol: String,
    pub timeframe: String,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

pub struct TickRepository {
    pool: Option<Pool<Postgres>>,
    event_bus: Option<Arc<EventBusPublisher>>,
}

impl TickRepository {
    pub fn new(pool: Option<Pool<Postgres>>, event_bus: Option<Arc<EventBusPublisher>>) -> Self {
        Self { pool, event_bus }
    }

    pub async fn save_tick(&self, record: &TickRecord) -> Result<(), sqlx::Error> {
        if let Some(pool) = &self.pool {
            sqlx::query(
                r#"
                INSERT INTO ticks (symbol, bid, ask, spread, timestamp_ms)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (symbol, timestamp_ms) DO NOTHING
                "#,
            )
            .bind(&record.symbol)
            .bind(record.bid)
            .bind(record.ask)
            .bind(record.spread)
            .bind(record.timestamp.timestamp_millis())
            .execute(pool)
            .await?;
        }

        if let Some(bus) = &self.event_bus {
            let tick_event = TickReceivedEvent {
                symbol: record.symbol.clone(),
                timestamp: None,
                bid: Some(Price {
                    value: record.bid.to_string(),
                    digits: 0,
                    currency: "USD".to_string(),
                }),
                ask: Some(Price {
                    value: record.ask.to_string(),
                    digits: 0,
                    currency: "USD".to_string(),
                }),
                bid_provider: "binance".to_string(),
                ask_provider: "binance".to_string(),
            };

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let event = Event {
                event_id: Some(Uuid {
                    value: uuid::Uuid::new_v4().as_bytes().to_vec(),
                }),
                spec_version: None,
                occurred_at: Some(Timestamp {
                    seconds: now.as_secs() as i64,
                    nanos: now.subsec_nanos() as i32,
                }),
                published_at: Some(Timestamp {
                    seconds: now.as_secs() as i64,
                    nanos: now.subsec_nanos() as i32,
                }),
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

    pub async fn load_ticks_ordered(
        &self,
        symbol: &str,
        from_sequence: i64,
        limit: i64,
    ) -> Result<Vec<TickRecord>, sqlx::Error> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                r#"
                SELECT symbol, bid, ask, spread, timestamp_ms 
                FROM ticks 
                WHERE symbol = $1 AND timestamp_ms >= $2 
                ORDER BY timestamp_ms ASC 
                LIMIT $3
                "#,
            )
            .bind(symbol)
            .bind(from_sequence)
            .bind(limit)
            .fetch_all(pool)
            .await?;

            use sqlx::Row;
            let records = rows
                .into_iter()
                .map(|row| {
                    let ts_ms: i64 = row.get("timestamp_ms");
                    TickRecord {
                        symbol: row.get("symbol"),
                        sequence: 0,
                        bid: row.get("bid"),
                        ask: row.get("ask"),
                        spread: row.get("spread"),
                        timestamp: chrono::DateTime::from_timestamp_millis(ts_ms).unwrap_or_default(),
                    }
                })
                .collect();
            Ok(records)
        } else {
            Ok(vec![])
        }
    }

    pub async fn count_ticks(&self, symbol: &str) -> Result<i64, sqlx::Error> {
        if let Some(pool) = &self.pool {
            let row = sqlx::query("SELECT COUNT(*) FROM ticks WHERE symbol = $1")
                .bind(symbol)
                .fetch_one(pool)
                .await?;
            use sqlx::Row;
            Ok(row.get::<i64, _>(0))
        } else {
            Ok(0)
        }
    }
}

pub struct CandleRepository {
    pool: Option<Pool<Postgres>>,
    event_bus: Option<Arc<EventBusPublisher>>,
}

impl CandleRepository {
    pub fn new(pool: Option<Pool<Postgres>>, event_bus: Option<Arc<EventBusPublisher>>) -> Self {
        Self { pool, event_bus }
    }

    pub async fn save_candle(&self, record: &CandleRecord) -> Result<(), sqlx::Error> {
        if let Some(pool) = &self.pool {
            sqlx::query(
                r#"
                INSERT INTO candles (symbol, timeframe, open_price, high_price, low_price, close_price, volume, open_time, close_time, is_closed, tick_count)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, 0)
                ON CONFLICT (symbol, timeframe, open_time) DO NOTHING
                "#
            )
            .bind(&record.symbol)
            .bind(&record.timeframe)
            .bind(record.open)
            .bind(record.high)
            .bind(record.low)
            .bind(record.close)
            .bind(record.volume)
            .bind(record.start_time)
            .bind(record.end_time)
            .execute(pool)
            .await?;
        }

        if let Some(bus) = &self.event_bus {
            use apex_protos::events::CandleClosedEvent;
            use apex_protos::common::{Timeframe, TimeUnit, Volume};

            // Parse timeframe (e.g. "M1", "H4")
            let (unit, value) = if record.timeframe.starts_with('M') {
                (TimeUnit::Minute, record.timeframe[1..].parse().unwrap_or(1))
            } else if record.timeframe.starts_with('H') {
                (TimeUnit::Hour, record.timeframe[1..].parse().unwrap_or(1))
            } else if record.timeframe.starts_with('D') {
                (TimeUnit::Day, record.timeframe[1..].parse().unwrap_or(1))
            } else {
                (TimeUnit::Minute, 1)
            };

            let tf = Timeframe {
                unit: unit.into(),
                value,
            };

            let candle_event = CandleClosedEvent {
                symbol: record.symbol.clone(),
                timeframe: Some(tf),
                close_time: Some(Timestamp {
                    seconds: record.end_time.timestamp(),
                    nanos: record.end_time.timestamp_subsec_nanos() as i32,
                }),
                open: Some(Price {
                    value: record.open.to_string(),
                    digits: 0,
                    currency: "USD".to_string(),
                }),
                high: Some(Price {
                    value: record.high.to_string(),
                    digits: 0,
                    currency: "USD".to_string(),
                }),
                low: Some(Price {
                    value: record.low.to_string(),
                    digits: 0,
                    currency: "USD".to_string(),
                }),
                close: Some(Price {
                    value: record.close.to_string(),
                    digits: 0,
                    currency: "USD".to_string(),
                }),
                volume: Some(Volume {
                    units: record.volume.to_string(),
                    lot_size: "1.0".to_string(),
                    fractional: true,
                }),
                tick_count: 0,
            };

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let event = Event {
                event_id: Some(Uuid {
                    value: uuid::Uuid::new_v4().as_bytes().to_vec(),
                }),
                spec_version: None,
                occurred_at: Some(Timestamp {
                    seconds: now.as_secs() as i64,
                    nanos: now.subsec_nanos() as i32,
                }),
                published_at: Some(Timestamp {
                    seconds: now.as_secs() as i64,
                    nanos: now.subsec_nanos() as i32,
                }),
                event_type: "CandleClosedEvent".to_string(),
                source_service: "market-data-engine".to_string(),
                topic: "market_data.candles".to_string(),
                correlation: None,
                causation_id: "".to_string(),
                deduplication_key: "".to_string(),
                payload: Some(Payload::CandleClosed(candle_event)),
                payload_hash: vec![],
            };

            if let Err(e) = bus.publish(event).await {
                tracing::warn!("Failed to publish CandleClosedEvent: {}", e);
            }
        }

        Ok(())
    }

    pub async fn load_candles_ordered(
        &self,
        symbol: &str,
        timeframe: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CandleRecord>, sqlx::Error> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                r#"
                SELECT symbol, timeframe, open_price as open, high_price as high, low_price as low, close_price as close, volume, open_time as start_time, close_time as end_time 
                FROM candles 
                WHERE symbol = $1 AND timeframe = $2 AND open_time >= $3 AND open_time <= $4 
                ORDER BY open_time ASC
                "#,
            )
            .bind(symbol)
            .bind(timeframe)
            .bind(from)
            .bind(to)
            .fetch_all(pool)
            .await?;

            use sqlx::Row;
            let records = rows
                .into_iter()
                .map(|row| CandleRecord {
                    symbol: row.get("symbol"),
                    timeframe: row.get("timeframe"),
                    open: row.get("open"),
                    high: row.get("high"),
                    low: row.get("low"),
                    close: row.get("close"),
                    volume: row.get("volume"),
                    start_time: row.get("start_time"),
                    end_time: row.get("end_time"),
                })
                .collect();
            Ok(records)
        } else {
            Ok(vec![])
        }
    }
}

pub struct MarketDataStore {
    pub pool: Option<Pool<Postgres>>,
    pub ticks: TickRepository,
    pub candles: CandleRepository,
    pub event_bus: Option<Arc<EventBusPublisher>>,
}

impl MarketDataStore {
    pub fn new(pool: Pool<Postgres>, event_bus: Option<Arc<EventBusPublisher>>) -> Self {
        let store = Self {
            pool: Some(pool.clone()),
            ticks: TickRepository::new(Some(pool.clone()), event_bus.clone()),
            candles: CandleRepository::new(Some(pool), event_bus.clone()),
            event_bus,
        };
        // Database tables managed via Prisma / Alembic elsewhere
        store
    }
}
