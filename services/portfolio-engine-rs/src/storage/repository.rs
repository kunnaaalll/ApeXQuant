use anyhow::Result;
use rust_decimal::Decimal;

use super::events::EventRecord;
use super::pg_store::PostgresPortfolioStore;
use super::snapshots::{SnapshotFrequency, SnapshotRecord};

/// PortfolioRepository provides a high-level API over the underlying store
/// enabling deterministic state reconstructions, snapshotting, and event publishing.
#[derive(Clone)]
pub struct PortfolioRepository {
    pub store: PostgresPortfolioStore,
}

impl PortfolioRepository {
    pub fn new(store: PostgresPortfolioStore) -> Self {
        Self { store }
    }

    /// Persist a single event and optionally a snapshot within the same transaction.
    /// Ensures 100% atomicity between the event log and the snapshot.
    pub async fn save_event_with_snapshot(
        &self,
        event: &EventRecord,
        snapshot: Option<&SnapshotRecord>,
    ) -> Result<()> {
        let mut tx = self.store.begin_transaction().await?;

        self.store.append_event(&mut tx, event).await?;

        if let Some(snap) = snapshot {
            self.store.append_snapshot(&mut tx, snap).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Save a snapshot directly. Useful for periodic rollup tasks (e.g. M5, H1).
    pub async fn save_snapshot(&self, snapshot: &SnapshotRecord) -> Result<()> {
        let mut tx = self.store.begin_transaction().await?;
        self.store.append_snapshot(&mut tx, snapshot).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Load the most recent snapshot for a specific aggregate and frequency.
    pub async fn load_latest_snapshot(
        &self,
        aggregate_id: &str,
        frequency: SnapshotFrequency,
    ) -> Result<Option<SnapshotRecord>> {
        self.store
            .load_latest_snapshot(aggregate_id, frequency)
            .await
    }

    /// Load a specific snapshot by its version number.
    pub async fn load_snapshot_by_version(
        &self,
        aggregate_id: &str,
        version: i64,
    ) -> Result<Option<SnapshotRecord>> {
        self.store
            .load_snapshot_by_version(aggregate_id, version)
            .await
    }

    /// Load the entire event history for an aggregate.
    pub async fn load_events(&self, aggregate_id: &str) -> Result<Vec<EventRecord>> {
        self.store.load_events(aggregate_id).await
    }

    /// Load events strictly greater than a specific version.
    /// This is used for replaying state on top of an existing snapshot.
    pub async fn load_events_since(
        &self,
        aggregate_id: &str,
        since_version: i64,
    ) -> Result<Vec<EventRecord>> {
        // We load all for now, but in production we'd filter in the database query.
        let mut events = self.store.load_events(aggregate_id).await?;
        events.retain(|e| e.version > since_version);
        Ok(events)
    }

    pub async fn load_events_since_time(
        &self,
        aggregate_id: &str,
        since_time: time::OffsetDateTime,
    ) -> Result<Vec<EventRecord>> {
        self.store
            .load_events_since_time(aggregate_id, since_time)
            .await
    }
}

#[derive(Clone)]
pub struct AllocationRepository {
    store: PostgresPortfolioStore,
}

impl AllocationRepository {
    pub fn new(store: PostgresPortfolioStore) -> Self {
        Self { store }
    }

    pub async fn save(&self, portfolio_id: &str, allocations: &serde_json::Value) -> Result<()> {
        self.store.save_allocation(portfolio_id, allocations).await
    }
}

#[derive(Clone)]
pub struct OptimizationRepository {
    store: PostgresPortfolioStore,
}

impl OptimizationRepository {
    pub fn new(store: PostgresPortfolioStore) -> Self {
        Self { store }
    }

    pub async fn save(
        &self,
        portfolio_id: &str,
        method: &str,
        weights: &serde_json::Value,
        expected_return: Decimal,
        estimated_volatility: Decimal,
        sharpe_ratio: Decimal,
    ) -> Result<()> {
        self.store
            .save_optimization(
                portfolio_id,
                method,
                weights,
                expected_return,
                estimated_volatility,
                sharpe_ratio,
            )
            .await
    }
}

#[derive(Clone)]
pub struct ExposureRepository {
    store: PostgresPortfolioStore,
}

impl ExposureRepository {
    pub fn new(store: PostgresPortfolioStore) -> Self {
        Self { store }
    }

    pub async fn save(
        &self,
        portfolio_id: &str,
        gross_exposure: Decimal,
        net_exposure: Decimal,
        long_exposure: Decimal,
        short_exposure: Decimal,
    ) -> Result<()> {
        self.store
            .save_exposure(
                portfolio_id,
                gross_exposure,
                net_exposure,
                long_exposure,
                short_exposure,
            )
            .await
    }
}

#[derive(Clone)]
pub struct HealthRepository {
    store: PostgresPortfolioStore,
}

impl HealthRepository {
    pub fn new(store: PostgresPortfolioStore) -> Self {
        Self { store }
    }

    pub async fn save(
        &self,
        portfolio_id: &str,
        health_score: i32,
        status: &str,
        breakdown: &serde_json::Value,
    ) -> Result<()> {
        self.store
            .save_health(portfolio_id, health_score, status, breakdown)
            .await
    }
}

#[derive(Clone)]
pub struct QualityRepository {
    store: PostgresPortfolioStore,
}

impl QualityRepository {
    pub fn new(store: PostgresPortfolioStore) -> Self {
        Self { store }
    }

    pub async fn save(
        &self,
        portfolio_id: &str,
        quality_score: Decimal,
        breakdown: &serde_json::Value,
    ) -> Result<()> {
        self.store
            .save_quality(portfolio_id, quality_score, breakdown)
            .await
    }
}

#[derive(Clone)]
pub struct CorrelationRepository {
    store: PostgresPortfolioStore,
}

impl CorrelationRepository {
    pub fn new(store: PostgresPortfolioStore) -> Self {
        Self { store }
    }

    pub async fn save(
        &self,
        portfolio_id: &str,
        window: &str,
        matrix_type: &str,
        identifiers: &serde_json::Value,
        data: &serde_json::Value,
    ) -> Result<()> {
        self.store
            .save_correlation(portfolio_id, window, matrix_type, identifiers, data)
            .await
    }
}
