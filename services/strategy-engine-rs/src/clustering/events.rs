use super::ClusterType;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterEvent {
    ClusterUpdated {
        cluster: ClusterType,
        confidence: Decimal,
    },
}
