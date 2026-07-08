use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::meta::strategy_state::StrategyState;
use crate::api::service::StrategyState as ApiStrategyState;

#[derive(Clone)]
pub struct StrategyEntry {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub state: StrategyState,
    pub internal_state: Arc<RwLock<ApiStrategyState>>,
}

#[derive(Clone)]
pub struct StrategyRegistry {
    strategies: Arc<RwLock<HashMap<Uuid, StrategyEntry>>>,
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, id: Uuid, name: String, version: String) {
        let entry = StrategyEntry {
            id,
            name,
            version,
            state: StrategyState::Normal,
            internal_state: Arc::new(RwLock::new(ApiStrategyState::new())),
        };
        let mut w = self.strategies.write().await;
        w.insert(id, entry);
    }

    pub async fn get(&self, id: &Uuid) -> Option<StrategyEntry> {
        let r = self.strategies.read().await;
        r.get(id).cloned()
    }

    pub async fn remove(&self, id: &Uuid) {
        let mut w = self.strategies.write().await;
        w.remove(id);
    }

    pub async fn update_state(&self, id: &Uuid, new_state: StrategyState) -> Result<(), String> {
        let mut w = self.strategies.write().await;
        if let Some(entry) = w.get_mut(id) {
            entry.state = new_state;
            Ok(())
        } else {
            Err("Strategy not found".to_string())
        }
    }
    
    pub async fn active_strategies(&self) -> Vec<StrategyEntry> {
        let r = self.strategies.read().await;
        r.values().cloned().collect()
    }
}
