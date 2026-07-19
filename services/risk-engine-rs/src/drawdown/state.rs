use super::DrawdownTracker;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DrawdownState {
    pub account_id: String,
    pub tracker: DrawdownTracker,
}

impl DrawdownState {
    pub fn new(account_id: String) -> Self {
        Self {
            account_id,
            tracker: DrawdownTracker::new(),
        }
    }
}
