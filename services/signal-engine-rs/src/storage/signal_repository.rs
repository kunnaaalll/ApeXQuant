use crate::error::Result;
use crate::signals::SignalResult;
use sqlx::{Pool, Sqlite};

/// Repository for persisting generated signals
pub struct SignalRepository {
    pool: Pool<Sqlite>,
}

impl SignalRepository {
    /// Create a new SignalRepository
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Initialize the database schema
    pub async fn initialize(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS signals (
                id TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                direction TEXT NOT NULL,
                confidence REAL NOT NULL,
                confluence_score REAL NOT NULL,
                entry_price TEXT NOT NULL,
                stop_loss TEXT,
                take_profit TEXT,
                regime TEXT NOT NULL,
                patterns TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_signals_symbol ON signals (symbol);
            CREATE INDEX IF NOT EXISTS idx_signals_timestamp ON signals (timestamp);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| crate::SignalEngineError::internal(format!("Database init error: {}", e)))?;

        Ok(())
    }

    /// Save a generated signal
    pub async fn save_signal(&self, signal: &SignalResult) -> Result<()> {
        let direction_str = match signal.direction {
            crate::signals::result::SignalDirection::Long => "LONG",
            crate::signals::result::SignalDirection::Short => "SHORT",
            crate::signals::result::SignalDirection::Neutral => "NEUTRAL",
        };

        let stop_loss_str = signal.stop_loss.map(|d| d.to_string());
        let take_profit_str = signal.take_profit.map(|d| d.to_string());
        let patterns_str = signal.patterns.join(",");
        // Use RFC3339 format explicitly
        let timestamp_str = signal
            .timestamp
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO signals (
                id, symbol, direction, confidence, confluence_score, entry_price, 
                stop_loss, take_profit, regime, patterns, timestamp
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(&signal.signal_id)
        .bind(&signal.symbol)
        .bind(direction_str)
        .bind(signal.confidence)
        .bind(signal.confluence_score)
        .bind(signal.entry_price.to_string())
        .bind(stop_loss_str)
        .bind(take_profit_str)
        .bind(&signal.regime)
        .bind(patterns_str)
        .bind(timestamp_str)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::SignalEngineError::internal(format!("Failed to save signal: {}", e)))?;

        Ok(())
    }
}
