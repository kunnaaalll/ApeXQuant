use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseAnalytics {
    // To be expanded in future phases
    pub _placeholder: bool,
}

impl Default for BaseAnalytics {
    fn default() -> Self {
        Self {
            _placeholder: false,
        }
    }
}
