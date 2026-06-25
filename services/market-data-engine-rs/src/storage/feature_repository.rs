use crate::features::FeatureSnapshot;
use sqlx::{Pool, Postgres};

pub struct FeatureStoreRepository {
    pool: Pool<Postgres>,
}

impl FeatureStoreRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn save_snapshot(&self, snapshot: &FeatureSnapshot) -> Result<(), sqlx::Error> {
        let json_features = serde_json::to_value(&snapshot.features).map_err(|_| sqlx::Error::Protocol("JSON serialization error".into()))?;
        let json_metadata = serde_json::to_value(&snapshot.metadata).map_err(|_| sqlx::Error::Protocol("JSON serialization error".into()))?;
        
        let window_str = format!("{:?}", snapshot.window);

        sqlx::query(
            r#"
            INSERT INTO feature_snapshots (symbol, window, metadata, features, timestamp)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(&snapshot.symbol)
        .bind(&window_str)
        .bind(&json_metadata)
        .bind(&json_features)
        .bind(snapshot.timestamp.timestamp_millis())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
