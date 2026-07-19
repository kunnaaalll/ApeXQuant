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
                INSERT INTO ticks (symbol, sequence, bid, ask, spread, timestamp)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (symbol, sequence) DO NOTHING
                "#,
            )
            .bind(&record.symbol)
            .bind(record.sequence)
            .bind(record.bid)
            .bind(record.ask)
            .bind(record.spread)
            .bind(record.timestamp)
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
                SELECT symbol, sequence, bid, ask, spread, timestamp 
                FROM ticks 
                WHERE symbol = $1 AND sequence >= $2 
                ORDER BY sequence ASC 
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
                .map(|row| TickRecord {
                    symbol: row.get("symbol"),
                    sequence: row.get("sequence"),
                    bid: row.get("bid"),
                    ask: row.get("ask"),
                    spread: row.get("spread"),
                    timestamp: row.get("timestamp"),
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
}

impl CandleRepository {
    pub fn new(pool: Option<Pool<Postgres>>) -> Self {
        Self { pool }
    }

    pub async fn save_candle(&self, record: &CandleRecord) -> Result<(), sqlx::Error> {
        if let Some(pool) = &self.pool {
            sqlx::query(
                r#"
                INSERT INTO candles (symbol, timeframe, open, high, low, close, volume, start_time, end_time)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (symbol, timeframe, start_time) DO NOTHING
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
                SELECT symbol, timeframe, open, high, low, close, volume, start_time, end_time 
                FROM candles 
                WHERE symbol = $1 AND timeframe = $2 AND start_time >= $3 AND start_time <= $4 
                ORDER BY start_time ASC
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
            candles: CandleRepository::new(Some(pool)),
            event_bus,
        };
        let p = store.pool.clone();
        if let Some(pool_ref) = p {
            tokio::spawn(async move {
                let _ = sqlx::query(
                    "CREATE TABLE IF NOT EXISTS ticks (
                        symbol VARCHAR(32) NOT NULL,
                        sequence BIGINT NOT NULL,
                        bid NUMERIC NOT NULL,
                        ask NUMERIC NOT NULL,
                        spread NUMERIC NOT NULL,
                        timestamp TIMESTAMPTZ NOT NULL,
                        PRIMARY KEY (symbol, sequence)
                    );",
                )
                .execute(&pool_ref)
                .await;

                let _ = sqlx::query(
                    "CREATE TABLE IF NOT EXISTS candles (
                        symbol VARCHAR(32) NOT NULL,
                        timeframe VARCHAR(16) NOT NULL,
                        open NUMERIC NOT NULL,
                        high NUMERIC NOT NULL,
                        low NUMERIC NOT NULL,
                        close NUMERIC NOT NULL,
                        volume NUMERIC NOT NULL,
                        start_time TIMESTAMPTZ NOT NULL,
                        end_time TIMESTAMPTZ NOT NULL,
                        PRIMARY KEY (symbol, timeframe, start_time)
                    );",
                )
                .execute(&pool_ref)
                .await;
            });
        }
        store
    }
}
