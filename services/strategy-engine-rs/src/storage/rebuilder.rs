use super::events::StrategyEventWrapper;
use serde::{Deserialize, Serialize};

pub trait Aggregatable: Sized + Default {
    type Snapshot: Serialize + for<'de> Deserialize<'de>;
    type Error;

    /// Apply an event to the aggregate in-place.
    fn apply_event(&mut self, event: &StrategyEventWrapper) -> Result<(), Self::Error>;

    /// Create a snapshot of the current state.
    fn snapshot(&self) -> Self::Snapshot;

    /// Restore the aggregate from a snapshot.
    fn restore(snapshot: Self::Snapshot) -> Result<Self, Self::Error>;
}

pub struct StrategyEventRebuilder;

impl StrategyEventRebuilder {
    /// Rebuilds an aggregate from an optional snapshot and a sequence of events.
    pub fn rebuild<A: Aggregatable>(
        snapshot: Option<A::Snapshot>,
        events: &[StrategyEventWrapper],
    ) -> Result<A, A::Error> {
        let mut aggregate = match snapshot {
            Some(snap) => A::restore(snap)?,
            None => A::default(),
        };

        for event in events {
            aggregate.apply_event(event)?;
        }

        Ok(aggregate)
    }
}
