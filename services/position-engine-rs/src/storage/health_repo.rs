use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct HealthRepository {
    pool: PgPool,
}

impl HealthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_health(
        &self,
        position_id: Uuid,
        health_score: Decimal,
        margin_utilization: Decimal,
        stop_distance: Decimal,
        liquidation_distance: Decimal,
        drawdown: Decimal,
        age_score: Decimal,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO position_health (
                position_id, health_score, margin_utilization, stop_distance, liquidation_distance, drawdown, age_score
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (position_id) DO UPDATE SET
                health_score = EXCLUDED.health_score,
                margin_utilization = EXCLUDED.margin_utilization,
                stop_distance = EXCLUDED.stop_distance,
                liquidation_distance = EXCLUDED.liquidation_distance,
                drawdown = EXCLUDED.drawdown,
                age_score = EXCLUDED.age_score,
                updated_at = NOW()
            "#
        )
        .bind(position_id)
        .bind(health_score)
        .bind(margin_utilization)
        .bind(stop_distance)
        .bind(liquidation_distance)
        .bind(drawdown)
        .bind(age_score)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
