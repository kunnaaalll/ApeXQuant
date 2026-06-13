//! Signal filters

pub mod quality;
pub mod regime;
pub mod session;
pub mod duplicates;

/// Filter result
#[derive(Debug, Clone)]
pub enum FilterResult {
    /// Signal passes filter
    Pass,
    /// Signal filtered out
    Reject(String),
}
