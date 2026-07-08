use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct LearningRepository {
    pool: PgPool,
}

impl LearningRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record_lesson(
        &self,
        lesson_id: Uuid,
        position_id: &str,
        signal_id: &str,
        strategy_id: &str,
        lesson_type: &str,
        category: &str,
        severity: f64,
        symbol: &str,
        market_regime: &str,
        gross_pnl: Decimal,
        net_pnl: Decimal,
        entry_efficiency: Decimal,
        exit_efficiency: Decimal,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO learning_lessons 
            (lesson_id, position_id, signal_id, strategy_id, lesson_type, category, severity, symbol, market_regime, gross_pnl, net_pnl, entry_efficiency, exit_efficiency)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#
        )
        .bind(lesson_id)
        .bind(position_id)
        .bind(signal_id)
        .bind(strategy_id)
        .bind(lesson_type)
        .bind(category)
        .bind(severity)
        .bind(symbol)
        .bind(market_regime)
        .bind(gross_pnl)
        .bind(net_pnl)
        .bind(entry_efficiency)
        .bind(exit_efficiency)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct MemoryRepository {
    pool: PgPool,
}

impl MemoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn update_memory(
        &self,
        strategy_id: &str,
        net_pnl: Decimal,
        is_winner: bool,
        regime_quality_delta: Decimal,
        execution_quality_delta: Decimal,
        ema_return: Decimal,
    ) -> Result<(), sqlx::Error> {
        let winning_inc = if is_winner { 1 } else { 0 };

        sqlx::query(
            r#"
            INSERT INTO learning_memory 
            (strategy_id, total_trades, winning_trades, regime_quality, execution_quality, ema_return, historical_sum_return)
            VALUES ($1, 1, $2, 1.0 + $3, 1.0 + $4, $5, $6)
            ON CONFLICT (strategy_id) DO UPDATE SET
                total_trades = learning_memory.total_trades + 1,
                winning_trades = learning_memory.winning_trades + $2,
                regime_quality = learning_memory.regime_quality + $3,
                execution_quality = learning_memory.execution_quality + $4,
                ema_return = $5,
                historical_sum_return = learning_memory.historical_sum_return + $6,
                updated_at = NOW()
            "#
        )
        .bind(strategy_id)
        .bind(winning_inc)
        .bind(regime_quality_delta)
        .bind(execution_quality_delta)
        .bind(ema_return)
        .bind(net_pnl)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_memory(&self, strategy_id: &str) -> Result<Option<StrategyMemoryRow>, sqlx::Error> {
        let row = sqlx::query_as::<_, StrategyMemoryRow>(
            r#"SELECT total_trades, winning_trades, regime_quality, execution_quality, ema_return, historical_sum_return, regime_transitions FROM learning_memory WHERE strategy_id = $1"#
        )
        .bind(strategy_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }
}

#[derive(sqlx::FromRow)]
pub struct StrategyMemoryRow {
    pub total_trades: i64,
    pub winning_trades: i64,
    pub regime_quality: Decimal,
    pub execution_quality: Decimal,
    pub ema_return: Decimal,
    pub historical_sum_return: Decimal,
    pub regime_transitions: i32,
}
