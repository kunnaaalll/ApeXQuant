use crate::positions::{PositionState, PositionTracker};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

pub struct PositionRepository {
    pool: PgPool,
}

impl PositionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_position(&self, tracker: &PositionTracker) -> Result<(), sqlx::Error> {
        let state_str = match tracker.state {
            PositionState::Opening => "opening",
            PositionState::Active => "active",
            PositionState::ScalingIn => "scaling_in",
            PositionState::ScalingOut => "scaling_out",
            PositionState::Reducing => "reducing",
            PositionState::Closing => "closing",
            PositionState::Closed => "closed",
            PositionState::Archived => "archived",
            PositionState::Invalid => "invalid",
        };

        let opened_dt: OffsetDateTime = tracker.opened_at.into();
        let updated_dt: OffsetDateTime = tracker.last_updated_at.into();
        let closed_dt: Option<OffsetDateTime> = tracker.closed_at.map(|c| c.into());

        sqlx::query(
            r#"
            INSERT INTO positions (
                position_id, symbol, side, state, 
                initial_volume, current_volume, entry_price, current_price, 
                stop_loss, take_profit, unrealized_pnl, realized_pnl,
                margin_used, commission, swap, leverage,
                opened_at, updated_at, closed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            ON CONFLICT (position_id) DO UPDATE SET
                state = EXCLUDED.state,
                current_volume = EXCLUDED.current_volume,
                current_price = EXCLUDED.current_price,
                stop_loss = EXCLUDED.stop_loss,
                take_profit = EXCLUDED.take_profit,
                unrealized_pnl = EXCLUDED.unrealized_pnl,
                realized_pnl = EXCLUDED.realized_pnl,
                margin_used = EXCLUDED.margin_used,
                commission = EXCLUDED.commission,
                swap = EXCLUDED.swap,
                updated_at = EXCLUDED.updated_at,
                closed_at = EXCLUDED.closed_at
            "#
        )
        .bind(tracker.position_id)
        .bind(&tracker.symbol)
        .bind(tracker.side.to_lowercase())
        .bind(state_str)
        .bind(tracker.initial_size)
        .bind(tracker.current_size)
        .bind(tracker.initial_entry_price)
        .bind(tracker.current_price)
        .bind(tracker.current_stop_loss)
        .bind(tracker.initial_take_profit)
        .bind(tracker.unrealized_pnl)
        .bind(tracker.realized_pnl)
        .bind(tracker.margin_used.unwrap_or(Decimal::ZERO))
        .bind(tracker.commission.unwrap_or(Decimal::ZERO))
        .bind(tracker.swap.unwrap_or(Decimal::ZERO))
        .bind(tracker.leverage.unwrap_or(Decimal::ONE))
        .bind(opened_dt)
        .bind(updated_dt)
        .bind(closed_dt)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_position(&self, id: Uuid) -> Result<Option<PositionTracker>, sqlx::Error> {
        let row_opt = sqlx::query(
            r#"
            SELECT position_id, symbol, side, state, 
                   initial_volume, current_volume, entry_price, current_price, 
                   stop_loss, take_profit, unrealized_pnl, realized_pnl,
                   margin_used, commission, swap, leverage,
                   opened_at, updated_at, closed_at
            FROM positions
            WHERE position_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row_opt {
            let position_id: Uuid = row.try_get("position_id")?;
            let state_str: String = row.try_get("state")?;
            let state = match state_str.as_str() {
                "opening" => PositionState::Opening,
                "active" => PositionState::Active,
                "scaling_in" => PositionState::ScalingIn,
                "scaling_out" => PositionState::ScalingOut,
                "reducing" => PositionState::Reducing,
                "closing" => PositionState::Closing,
                "closed" => PositionState::Closed,
                "archived" => PositionState::Archived,
                _ => PositionState::Invalid,
            };

            let opened_dt: OffsetDateTime = row.try_get("opened_at")?;
            let updated_dt: OffsetDateTime = row.try_get("updated_at")?;
            let closed_dt_opt: Option<OffsetDateTime> = row.try_get("closed_at")?;

            let mut tracker = PositionTracker::new(
                position_id,
                row.try_get("symbol")?,
                row.try_get("side")?,
                row.try_get("initial_volume")?,
                row.try_get("entry_price")?,
            );
            tracker.state = state;
            tracker.current_size = row.try_get("current_volume")?;
            tracker.current_price = row.try_get("current_price")?;
            tracker.current_stop_loss = row.try_get("stop_loss")?;
            tracker.initial_take_profit = row.try_get("take_profit")?;
            tracker.initial_stop_loss = row.try_get("stop_loss")?;
            tracker.unrealized_pnl = row.try_get("unrealized_pnl")?;
            tracker.realized_pnl = row.try_get("realized_pnl")?;
            tracker.margin_used = row.try_get("margin_used").ok();
            tracker.commission = row.try_get("commission").ok();
            tracker.swap = row.try_get("swap").ok();
            tracker.leverage = row.try_get("leverage").ok();
            tracker.opened_at = opened_dt.into();
            tracker.last_updated_at = updated_dt.into();
            tracker.closed_at = closed_dt_opt.map(|dt| dt.into());

            Ok(Some(tracker))
        } else {
            Ok(None)
        }
    }
}
