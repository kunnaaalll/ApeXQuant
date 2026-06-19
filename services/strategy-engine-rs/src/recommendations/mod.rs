pub mod events;
pub mod reason_codes;
pub mod recommendation_engine;
pub mod snapshot;

#[cfg(test)]
mod tests;

pub use events::RecommendationEvent;
pub use reason_codes::ReasonCode;
pub use recommendation_engine::{Recommendation, RecommendationAction, RecommendationEngine};
pub use snapshot::RecommendationSnapshot;
