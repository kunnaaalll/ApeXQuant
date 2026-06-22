use crate::storage::events::ExecutionEventWrapper;
use serde_json::Value;

pub trait Aggregatable {
    fn apply_event(&mut self, event: &ExecutionEventWrapper);
    fn snapshot(&self) -> Value;
    fn restore_snapshot(&mut self, payload: Value);
}
