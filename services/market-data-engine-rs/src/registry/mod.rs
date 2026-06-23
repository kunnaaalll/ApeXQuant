use std::collections::HashMap;
use crate::connectors::MarketDataConnector;
use crate::health::FeedHealthGrade;

pub struct FeedRegistry {
    feeds: HashMap<String, Box<dyn MarketDataConnector>>,
}

impl Default for FeedRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedRegistry {
    pub fn new() -> Self {
        Self {
            feeds: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: String, connector: Box<dyn MarketDataConnector>) {
        self.feeds.insert(id, connector);
    }

    pub fn remove(&mut self, id: &str) -> Option<Box<dyn MarketDataConnector>> {
        self.feeds.remove(id)
    }

    pub fn lookup(&self, id: &str) -> Option<&(dyn MarketDataConnector + 'static)> {
        self.feeds.get(id).map(|b| b.as_ref())
    }

    pub fn lookup_mut(&mut self, id: &str) -> Option<&mut (dyn MarketDataConnector + 'static)> {
        self.feeds.get_mut(id).map(|b| b.as_mut())
    }

    pub fn health_summary(&self) -> HashMap<String, FeedHealthGrade> {
        let mut summary = HashMap::new();
        for (id, connector) in &self.feeds {
            summary.insert(id.clone(), connector.health());
        }
        summary
    }
}
