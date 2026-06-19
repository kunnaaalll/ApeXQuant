use crate::storage::events::{EventRecord, PortfolioEventWrapper};

pub trait Aggregatable {
    fn apply(&mut self, event: &PortfolioEventWrapper);
}

pub struct RiskEventRebuilder;

impl RiskEventRebuilder {
    pub fn rebuild<S>(initial_state: S, events: &[EventRecord]) -> S
    where
        S: Aggregatable,
    {
        let mut state = initial_state;
        for event in events {
            state.apply(&event.payload);
        }
        state
    }
}
