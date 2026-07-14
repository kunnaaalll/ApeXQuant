use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureWindow {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    OneDay,
}

impl FeatureWindow {
    pub fn to_seconds(&self) -> u64 {
        match self {
            FeatureWindow::OneMinute => 60,
            FeatureWindow::FiveMinutes => 300,
            FeatureWindow::FifteenMinutes => 900,
            FeatureWindow::OneHour => 3600,
            FeatureWindow::FourHours => 14400,
            FeatureWindow::OneDay => 86400,
        }
    }
}

pub struct FeatureStore {
    pool: PgPool,
}

impl FeatureStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_feature(&self, asset_id: &str, window: FeatureWindow, timestamp: u64, feature_data: &serde_json::Value) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO features (asset_id, window_seconds, timestamp, data)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (asset_id, window_seconds, timestamp) 
            DO UPDATE SET data = EXCLUDED.data
            "#
        )
        .bind(asset_id)
        .bind(window.to_seconds() as i64)
        .bind(timestamp as i64)
        .bind(feature_data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_feature(&self, asset_id: &str, window: FeatureWindow, timestamp: u64) -> Result<Option<serde_json::Value>> {
        let record = sqlx::query(
            r#"
            SELECT data FROM features
            WHERE asset_id = $1 AND window_seconds = $2 AND timestamp = $3
            "#
        )
        .bind(asset_id)
        .bind(window.to_seconds() as i64)
        .bind(timestamp as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|r| r.get("data")))
    }
}
