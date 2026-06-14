use crate::shadow::{ShadowEvent, ShadowSnapshot};

/// An interface for Shadow Storage.
/// Must be append-only, replayable, and deterministically serializable.
pub trait ShadowStorage: Send + Sync {
    /// Append a new comparison event
    fn append_event(&self, event: ShadowEvent) -> Result<(), StorageError>;

    /// Store a snapshot
    fn store_snapshot(&self, snapshot: ShadowSnapshot) -> Result<(), StorageError>;

    /// Fetch events for a given time window or specific run
    fn fetch_events(&self, start_time: chrono::DateTime<chrono::Utc>, end_time: chrono::DateTime<chrono::Utc>) -> Result<Vec<ShadowEvent>, StorageError>;
}

#[derive(Debug)]
pub enum StorageError {
    ConnectionFailed,
    SerializationFailed,
    AppendFailed,
    ReadFailed,
}

// In real implementation, this would connect to Postgres or SQLite using sqlx or similar.
pub struct PgShadowStorage {
    // pool: sqlx::PgPool,
}

impl PgShadowStorage {
    pub fn new() -> Self {
        Self {}
    }
}

impl ShadowStorage for PgShadowStorage {
    fn append_event(&self, _event: ShadowEvent) -> Result<(), StorageError> {
        // sqlx append query
        Ok(())
    }

    fn store_snapshot(&self, _snapshot: ShadowSnapshot) -> Result<(), StorageError> {
        // sqlx insert snapshot
        Ok(())
    }

    fn fetch_events(&self, _start_time: chrono::DateTime<chrono::Utc>, _end_time: chrono::DateTime<chrono::Utc>) -> Result<Vec<ShadowEvent>, StorageError> {
        Ok(vec![])
    }
}
