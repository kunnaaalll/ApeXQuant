use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Exceptional,
    Strong,
    Normal,
    Weak,
    Negative,
}
