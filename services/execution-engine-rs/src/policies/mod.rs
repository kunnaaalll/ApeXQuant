use thiserror::Error;

pub mod fok;
pub mod gtc;
pub mod ioc;
pub mod limit;
pub mod market;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyState {
    New,
    Active,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PolicyError {
    #[error("Invalid state transition from {from:?} to {to:?} for policy {policy}")]
    InvalidTransition {
        policy: &'static str,
        from: PolicyState,
        to: PolicyState,
    },
}
