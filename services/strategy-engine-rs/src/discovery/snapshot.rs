use super::{DeteriorationState, EdgeState, VelocityEngine, VelocityType};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoverySnapshot {
    pub edge_state: EdgeState,
    pub velocity_engines: HashMap<VelocityType, VelocityEngine>,
    pub deterioration_state: DeteriorationState,
}

impl DiscoverySnapshot {
    pub fn new(
        edge_state: EdgeState,
        velocity_engines: HashMap<VelocityType, VelocityEngine>,
        deterioration_state: DeteriorationState,
    ) -> Self {
        Self {
            edge_state,
            velocity_engines,
            deterioration_state,
        }
    }
}
