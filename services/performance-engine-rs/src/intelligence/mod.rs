pub mod degradation_intelligence;
pub mod edge_intelligence;
pub mod expectancy_intelligence;
pub mod pattern_intelligence;
pub mod recommendation;

pub use degradation_intelligence::*;
pub use edge_intelligence::*;
pub use expectancy_intelligence::*;
pub use pattern_intelligence::*;
pub use recommendation::*;

#[cfg(test)]
mod tests;
