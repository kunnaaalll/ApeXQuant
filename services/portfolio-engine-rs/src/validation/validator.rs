use sqlx::{PgPool, Row};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct ReplayResult {
    pub events_processed: usize,
    pub drift_detected: bool,
    pub exact_match: bool,
}

pub struct ReplayValidator;

impl Default for ReplayValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayValidator {
    pub fn new() -> Self {
        Self
    }

    pub async fn validate_replay(
        &self,
        aggregate_id: &str,
        pool: &PgPool,
        current_state: &crate::portfolio::state::PortfolioState,
    ) -> Result<ReplayResult, String> {
        let records = sqlx::query(
            r#"
            SELECT payload, timestamp
            FROM portfolio_events
            WHERE aggregate_id = $1
            ORDER BY version ASC
            "#
        )
        .bind(aggregate_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut replayed_state = crate::portfolio::state::PortfolioState::new();
        let events_processed = records.len();

        for row in &records {
            let payload_json: serde_json::Value = row.try_get("payload").map_err(|e| e.to_string())?;
            let event: crate::portfolio::events::PortfolioEvent = serde_json::from_value(payload_json).map_err(|e| e.to_string())?;
            let ts: OffsetDateTime = row.try_get("timestamp").map_err(|e| e.to_string())?;
            
            replayed_state.apply_event(&event, ts).map_err(|e| format!("{:?}", e))?;
        }

        let exact_match = replayed_state.balance == current_state.balance &&
                           replayed_state.equity == current_state.equity &&
                           replayed_state.used_margin == current_state.used_margin &&
                           replayed_state.exposure == current_state.exposure;

        Ok(ReplayResult {
            events_processed,
            drift_detected: !exact_match,
            exact_match,
        })
    }
}
