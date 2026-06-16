use crate::shadow::statistics::ShadowStatistics;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ShadowStorage {
    statistics: Arc<RwLock<ShadowStatistics>>,
}

impl ShadowStorage {
    pub fn new() -> Self {
        Self {
            statistics: Arc::new(RwLock::new(ShadowStatistics::new())),
        }
    }

    pub async fn load_statistics(&self) -> ShadowStatistics {
        let stats = self.statistics.read().await;
        stats.clone()
    }

    pub async fn save_statistics(&self, new_stats: ShadowStatistics) {
        let mut stats = self.statistics.write().await;
        *stats = new_stats;
    }
}

impl Default for ShadowStorage {
    fn default() -> Self {
        Self::new()
    }
}
