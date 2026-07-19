use crate::positions::PositionTracker;
use crate::storage::{
    AnalyticsRepository, EventsRepository, HealthRepository, PositionRepository, SnapshotRepository,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresStore {
    pub positions: PositionRepository,
    pub snapshots: SnapshotRepository,
    pub analytics: AnalyticsRepository,
    pub health: HealthRepository,
    pub events: EventsRepository,
}

impl PostgresStore {
    pub fn new(pool: PgPool) -> Self {
        Self {
            positions: PositionRepository::new(pool.clone()),
            snapshots: SnapshotRepository::new(pool.clone()),
            analytics: AnalyticsRepository::new(pool.clone()),
            health: HealthRepository::new(pool.clone()),
            events: EventsRepository::new(pool.clone()),
        }
    }

    pub async fn save_position(&self, tracker: &PositionTracker) -> Result<(), sqlx::Error> {
        self.positions.save_position(tracker).await
    }

    pub async fn get_position(&self, id: Uuid) -> Result<Option<PositionTracker>, sqlx::Error> {
        self.positions.get_position(id).await
    }
}
