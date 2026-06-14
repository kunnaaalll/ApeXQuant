pub mod edge_drift;
pub mod expectancy_drift;
pub mod stability_drift;

pub use edge_drift::*;
pub use expectancy_drift::*;
pub use stability_drift::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftState {
    Improving,
    Stable,
    Weakening,
    Critical,
}

#[cfg(test)]
mod tests;
