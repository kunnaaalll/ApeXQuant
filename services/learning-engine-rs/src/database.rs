use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct LearningRepository {
    pool: PgPool,
}

pub struct RecordLessonParams<'a> {
    pub lesson_id: Uuid,
    pub position_id: &'a str,
    pub signal_id: &'a str,
    pub strategy_id: &'a str,
    pub lesson_type: &'a str,
    pub category: &'a str,
    pub severity: f64,
    pub symbol: &'a str,
    pub market_regime: &'a str,
    pub gross_pnl: Decimal,
    pub net_pnl: Decimal,
    pub entry_efficiency: Decimal,
    pub exit_efficiency: Decimal,
}

impl LearningRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Persist a learning event record to the feature store.
    /// Called for every event consumed from the event bus, regardless of type.
    #[allow(clippy::too_many_arguments)]
    pub async fn record_event(
        &self,
        event_type: &str,
        topic: &str,
        strategy_id: &str,
        symbol: &str,
        net_pnl: Decimal,
        gross_pnl: Decimal,
        label_is_winner: Option<bool>,
        features: serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        let raw_payload = features.clone();
        sqlx::query(
            r#"
            INSERT INTO learning_event_records
                (event_type, topic, strategy_id, symbol, net_pnl, gross_pnl, label_is_winner, features, raw_payload)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(event_type)
        .bind(topic)
        .bind(strategy_id)
        .bind(symbol)
        .bind(net_pnl)
        .bind(gross_pnl)
        .bind(label_is_winner)
        .bind(&features)
        .bind(&raw_payload)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn record_lesson(&self, params: RecordLessonParams<'_>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO learning_lessons 
            (lesson_id, position_id, signal_id, strategy_id, lesson_type, category, severity, symbol, market_regime, gross_pnl, net_pnl, entry_efficiency, exit_efficiency)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#
        )
        .bind(params.lesson_id)
        .bind(params.position_id)
        .bind(params.signal_id)
        .bind(params.strategy_id)
        .bind(params.lesson_type)
        .bind(params.category)
        .bind(params.severity)
        .bind(params.symbol)
        .bind(params.market_regime)
        .bind(params.gross_pnl)
        .bind(params.net_pnl)
        .bind(params.entry_efficiency)
        .bind(params.exit_efficiency)
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

    pub async fn get_memory(
        &self,
        strategy_id: &str,
    ) -> Result<Option<StrategyMemoryRow>, sqlx::Error> {
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
