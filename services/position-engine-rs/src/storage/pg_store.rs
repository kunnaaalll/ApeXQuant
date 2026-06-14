use crate::positions::PositionTracker;
use sqlx::PgPool;
use uuid::Uuid;

/// Handles Postgres persistence for the Position Engine.
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_position(&self, tracker: &PositionTracker) -> Result<(), sqlx::Error> {
        // TODO: Implement actual SQL insert/update query
        // Expected table: positions
        Ok(())
    }

    pub async fn get_position(&self, id: Uuid) -> Result<Option<PositionTracker>, sqlx::Error> {
        // TODO: Implement actual SQL select query
        Ok(None)
    }
}
