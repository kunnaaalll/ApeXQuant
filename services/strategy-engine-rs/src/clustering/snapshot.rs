use super::cluster_state::ClusterState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterSnapshot {
    pub state: ClusterState,
}

impl ClusterSnapshot {
    pub fn new(state: ClusterState) -> Self {
        Self { state }
    }
}
