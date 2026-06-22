use sqlx::{Postgres, Transaction};
use crate::storage::events::EventRecord;
use crate::storage::snapshots::SnapshotRecord;
use crate::storage::pg_store::PgStore;
use crate::storage::StorageError;

pub struct ExecutionTransaction<'a> {
    tx: Transaction<'a, Postgres>,
}

impl<'a> ExecutionTransaction<'a> {
    pub async fn begin(store: &'a PgStore) -> Result<Self, StorageError> {
        let tx = store.begin_transaction().await?;
        Ok(Self { tx })
    }

    pub async fn save_event(&mut self, event: &EventRecord) -> Result<(), StorageError> {
        PgStore::append_event(&mut self.tx, event).await?;
        Ok(())
    }

    pub async fn save_snapshot(&mut self, snapshot: &SnapshotRecord) -> Result<(), StorageError> {
        PgStore::append_snapshot(&mut self.tx, snapshot).await?;
        Ok(())
    }

    pub async fn commit(self) -> Result<(), StorageError> {
        self.tx.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), StorageError> {
        self.tx.rollback().await?;
        Ok(())
    }
}
