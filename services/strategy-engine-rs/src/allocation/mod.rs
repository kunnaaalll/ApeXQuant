pub mod allocation_engine;
pub mod allocation_state;
pub mod events;
pub mod snapshot;

#[cfg(test)]
mod tests;

pub use allocation_engine::AllocationEngine;
pub use allocation_state::{AllocationState, ExposureState};
pub use events::AllocationEvent;
pub use snapshot::AllocationSnapshot;
