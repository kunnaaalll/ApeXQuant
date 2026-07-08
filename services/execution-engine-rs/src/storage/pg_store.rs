use crate::storage::events::EventRecord;
use crate::storage::snapshots::SnapshotRecord;
use crate::storage::StorageError;
use sqlx::{Pool, Postgres, Transaction};

pub struct PgStore {
    pool: Pool<Postgres>,
}

impl PgStore {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    pub async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>, StorageError> {
        Ok(self.pool.begin().await?)
    }

    pub async fn append_event(
        tx: &mut Transaction<'_, Postgres>,
        event: &EventRecord,
    ) -> Result<(), sqlx::Error> {
        let payload_json =
            serde_json::to_value(&event.payload).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let inner_val = match &event.payload {
            crate::storage::events::ExecutionEventWrapper::OrderEvent(val) => val,
            crate::storage::events::ExecutionEventWrapper::FillEvent(val) => val,
            crate::storage::events::ExecutionEventWrapper::PositionEvent(val) => val,
            crate::storage::events::ExecutionEventWrapper::ExecutionRiskEvent(val) => val,
            crate::storage::events::ExecutionEventWrapper::SmartExecutionEvent(val) => val,
            crate::storage::events::ExecutionEventWrapper::MicrostructureEvent(val) => val,
            crate::storage::events::ExecutionEventWrapper::BrokerEvent(val) => val,
            crate::storage::events::ExecutionEventWrapper::ShadowEvent(val) => val,
        };

        let val_to_decimal = |val: &serde_json::Value| -> Option<rust_decimal::Decimal> {
            match val {
                serde_json::Value::Number(num) => {
                    if let Some(f) = num.as_f64() {
                        use rust_decimal::prelude::FromPrimitive;
                        rust_decimal::Decimal::from_f64(f)
                    } else {
                        num.as_i64().map(rust_decimal::Decimal::from)
                    }
                }
                serde_json::Value::String(s) => s.parse::<rust_decimal::Decimal>().ok(),
                _ => None,
            }
        };

        let order_id = inner_val
            .get("order_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let position_id = inner_val
            .get("position_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let broker_ticket = inner_val
            .get("broker_ticket")
            .or_else(|| inner_val.get("ticket"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let symbol = inner_val
            .get("symbol")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let side = inner_val
            .get("side")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let order_type = inner_val
            .get("order_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let volume = inner_val.get("volume").and_then(val_to_decimal);
        let requested_price = inner_val
            .get("price")
            .or_else(|| inner_val.get("requested_price"))
            .and_then(val_to_decimal);
        let fill_price = inner_val.get("fill_price").and_then(val_to_decimal);
        let stop_loss = inner_val.get("stop_loss").and_then(val_to_decimal);
        let take_profit = inner_val.get("take_profit").and_then(val_to_decimal);
        let slippage_points = inner_val
            .get("slippage_points")
            .or_else(|| inner_val.get("slippage"))
            .and_then(val_to_decimal);
        let latency_ms = inner_val
            .get("latency_ms")
            .or_else(|| inner_val.get("latency"))
            .and_then(val_to_decimal);

        let broker_retcode = inner_val
            .get("broker_retcode")
            .or_else(|| inner_val.get("retcode"))
            .and_then(|v| v.as_i64())
            .map(|i| i as i32);
        let broker_comment = inner_val
            .get("broker_comment")
            .or_else(|| inner_val.get("comment"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let pnl = inner_val
            .get("pnl")
            .or_else(|| inner_val.get("floating_pnl"))
            .and_then(val_to_decimal);
        let swap = inner_val.get("swap").and_then(val_to_decimal);
        let commission = inner_val.get("commission").and_then(val_to_decimal);

        sqlx::query(
            r#"
            INSERT INTO execution_events (
                aggregate_id, sequence_number, event_type, timestamp, payload, version,
                order_id, position_id, broker_ticket, symbol, side, order_type,
                volume, requested_price, fill_price, stop_loss, take_profit,
                slippage_points, latency_ms, broker_retcode, broker_comment,
                pnl, swap, commission
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)
            "#
        )
        .bind(event.aggregate_id)
        .bind(event.sequence_number as i64)
        .bind(&event.event_type)
        .bind(event.timestamp)
        .bind(payload_json)
        .bind(event.version as i32)
        .bind(order_id)
        .bind(position_id)
        .bind(broker_ticket)
        .bind(symbol)
        .bind(side)
        .bind(order_type)
        .bind(volume)
        .bind(requested_price)
        .bind(fill_price)
        .bind(stop_loss)
        .bind(take_profit)
        .bind(slippage_points)
        .bind(latency_ms)
        .bind(broker_retcode)
        .bind(broker_comment)
        .bind(pnl)
        .bind(swap)
        .bind(commission)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn load_events(
        &self,
        aggregate_id: uuid::Uuid,
        after_sequence: u64,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        use sqlx::Row;

        let records = sqlx::query(
            r#"
            SELECT aggregate_id, sequence_number, event_type, timestamp, payload, version
            FROM execution_events
            WHERE aggregate_id = $1 AND sequence_number > $2
            ORDER BY sequence_number ASC
            "#,
        )
        .bind(aggregate_id)
        .bind(after_sequence as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(records.len());
        for r in records {
            let payload_value: serde_json::Value = r.get("payload");
            let payload = serde_json::from_value(payload_value)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let timestamp_dt: time::OffsetDateTime = r.get("timestamp");

            events.push(EventRecord {
                aggregate_id: r.get("aggregate_id"),
                sequence_number: r.get::<i64, _>("sequence_number") as u64,
                event_type: r.get("event_type"),
                timestamp: timestamp_dt,
                payload,
                version: r.get::<i32, _>("version") as u32,
            });
        }

        Ok(events)
    }

    pub async fn append_snapshot(
        tx: &mut Transaction<'_, Postgres>,
        snapshot: &SnapshotRecord,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO execution_snapshots (aggregate_id, snapshot_version, sequence_number, payload)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(snapshot.aggregate_id)
        .bind(snapshot.snapshot_version as i32)
        .bind(snapshot.sequence_number as i64)
        .bind(&snapshot.payload)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn load_latest_snapshot(
        &self,
        aggregate_id: uuid::Uuid,
    ) -> Result<Option<SnapshotRecord>, sqlx::Error> {
        use sqlx::Row;

        let record = sqlx::query(
            r#"
            SELECT aggregate_id, snapshot_version, sequence_number, payload
            FROM execution_snapshots
            WHERE aggregate_id = $1
            ORDER BY sequence_number DESC
            LIMIT 1
            "#,
        )
        .bind(aggregate_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|r| {
            let payload: serde_json::Value = r.get("payload");
            SnapshotRecord {
                aggregate_id: r.get("aggregate_id"),
                snapshot_version: r.get::<i32, _>("snapshot_version") as u32,
                sequence_number: r.get::<i64, _>("sequence_number") as u64,
                payload,
            }
        }))
    }
}
