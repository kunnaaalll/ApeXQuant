pub mod edge_discovery;
pub mod opportunity_detector;
pub mod deterioration_detector;

pub use edge_discovery::{EdgeDiscovery, DiscoveryState};
pub use opportunity_detector::{OpportunityDetector, Opportunity, OpportunityState};
pub use deterioration_detector::{DeteriorationDetector, DeteriorationState};
