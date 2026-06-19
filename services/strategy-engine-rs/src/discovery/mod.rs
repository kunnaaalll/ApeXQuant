pub mod deterioration_detector;
pub mod edge_discovery;
pub mod events;
pub mod snapshot;
pub mod velocity;

#[cfg(test)]
mod tests;

pub use deterioration_detector::{DeteriorationDetector, DeteriorationState};
pub use edge_discovery::{EdgeDiscovery, EdgeState};
pub use events::DiscoveryEvent;
pub use snapshot::DiscoverySnapshot;
pub use velocity::{VelocityEngine, VelocityState, VelocityType};
