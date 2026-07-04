use crate::filters::FilterResult;
use crate::signals::SignalResult;
use std::collections::HashSet;
use std::sync::Mutex;

pub struct DuplicateFilter {
    recent_signals: Mutex<HashSet<String>>,
}

impl DuplicateFilter {
    pub fn new() -> Self {
        Self {
            recent_signals: Mutex::new(HashSet::new()),
        }
    }

    /// Reject if same symbol signal direction was recently emitted
    pub fn filter(&self, signal: &SignalResult) -> FilterResult {
        let mut recent = match self.recent_signals.lock() {
            Ok(guard) => guard,
            Err(_) => return FilterResult::Pass, // fallback on poisoning
        };

        let key = format!("{}-{:?}", signal.symbol, signal.direction);
        if recent.contains(&key) {
            FilterResult::Reject(format!(
                "Rejecting signal {} as duplicate of recent signal key {}",
                signal.signal_id, key
            ))
        } else {
            recent.insert(key);
            FilterResult::Pass
        }
    }
}

impl Default for DuplicateFilter {
    fn default() -> Self {
        Self::new()
    }
}
