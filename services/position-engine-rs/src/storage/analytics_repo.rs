use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct AnalyticsRepository {
    pool: PgPool,
}

impl AnalyticsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_analytics(
        &self,
        position_id: Uuid,
        holding_efficiency: Decimal,
        time_efficiency: Decimal,
        profit_velocity: Decimal,
        drawdown_duration: i64,
        recovery_time: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO position_analytics (
                position_id, holding_efficiency, time_efficiency, profit_velocity, drawdown_duration, recovery_time
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (position_id) DO UPDATE SET
                holding_efficiency = EXCLUDED.holding_efficiency,
                time_efficiency = EXCLUDED.time_efficiency,
                profit_velocity = EXCLUDED.profit_velocity,
                drawdown_duration = EXCLUDED.drawdown_duration,
                recovery_time = EXCLUDED.recovery_time,
                updated_at = NOW()
            "#
        )
        .bind(position_id)
        .bind(holding_efficiency)
        .bind(time_efficiency)
        .bind(profit_velocity)
        .bind(drawdown_duration)
        .bind(recovery_time)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
