use super::{DeteriorationState, EdgeState, VelocityState, VelocityType};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryEvent {
    EdgeStateChanged {
        old_state: EdgeState,
        new_state: EdgeState,
        edge_delta: Decimal,
        expectancy_delta: Decimal,
        confidence_delta: Decimal,
    },
    VelocityChanged {
        velocity_type: VelocityType,
        old_state: VelocityState,
        new_state: VelocityState,
        current_velocity: Decimal,
    },
    DeteriorationChanged {
        old_state: DeteriorationState,
        new_state: DeteriorationState,
    },
}
