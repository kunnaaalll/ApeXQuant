pub mod aggregate;
pub mod consistency;
pub mod events;
pub mod pg_store;
pub mod rebuilder;
pub mod repository;
pub mod sequence;
pub mod serializer;
pub mod snapshots;
pub mod transactions;
pub mod versioning;

#[cfg(test)]
pub mod tests;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Sequence violation: expected {expected}, got {actual}")]
    SequenceViolation { expected: u64, actual: u64 },

    #[error("Version error: invalid version transition")]
    VersionError,

    #[error("Consistency error: {0}")]
    ConsistencyError(String),

    #[error("Snapshot not found")]
    SnapshotNotFound,
}
