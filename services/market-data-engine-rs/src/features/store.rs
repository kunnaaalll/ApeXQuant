use super::types::{FeatureSnapshot, FeatureWindow};
use std::collections::HashMap;

#[derive(Default)]
pub struct FeatureStore {
    snapshots: HashMap<String, HashMap<FeatureWindow, FeatureSnapshot>>,
}

impl FeatureStore {
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
        }
    }

    pub fn insert(&mut self, snapshot: FeatureSnapshot) {
        let windows = self.snapshots.entry(snapshot.symbol.clone()).or_default();
        windows.insert(snapshot.window, snapshot);
    }

    pub fn get(&self, symbol: &str, window: FeatureWindow) -> Option<&FeatureSnapshot> {
        self.snapshots.get(symbol).and_then(|w| w.get(&window))
    }
}
