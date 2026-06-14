use std::future::Future;
use std::pin::Pin;

use sqlx::PgPool;

use crate::{RiskAssessment, TradeResult, error::RiskError};
use super::ShadowStorage;

/// Postgres implementation of ShadowStorage
/// 
/// Automatically spawns background tasks for inserts to avoid blocking the critical path.
#[derive(Clone)]
pub struct PostgresShadowStorage {
    pool: PgPool,
}

impl PostgresShadowStorage {
    /// Create a new PostgresShadowStorage
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ShadowStorage for PostgresShadowStorage {
    fn store_comparison<'a>(
        &'a self,
        trades: &'a [TradeResult],
        assessment: &'a RiskAssessment,
    ) -> Pin<Box<dyn Future<Output = Result<(), RiskError>> + Send + 'a>> {
        let pool = self.pool.clone();
        
        // Clone data to move into static lifetime background task
        let trades_cloned = trades.to_vec();
        let assessment_cloned = assessment.clone();
        
        Box::pin(async move {
            tokio::spawn(async move {
                let trades_json = serde_json::to_value(&trades_cloned).unwrap_or(serde_json::Value::Null);
                let assessment_json = serde_json::to_value(&assessment_cloned).unwrap_or(serde_json::Value::Null);
                
                let result = sqlx::query(
                    r#"
                    INSERT INTO risk_shadow_assessments 
                    (timestamp, trades_context, risk_assessment, approved, lot_size, latency_us)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#
                )
                .bind(assessment_cloned.timestamp)
                .bind(trades_json)
                .bind(assessment_json)
                .bind(assessment_cloned.approved)
                .bind(assessment_cloned.lot_size)
                .bind(assessment_cloned.latency_us as i64)
                .execute(&pool)
                .await;
                
                if let Err(e) = result {
                    tracing::error!("Failed to store shadow assessment in Postgres: {}", e);
                }
            });
            
            // Return Ok immediately to unblock critical path
            Ok(())
        })
    }
}
