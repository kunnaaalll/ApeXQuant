//! Confidence calculation

pub mod calculator;
pub mod factors;
pub mod decay;
pub mod bayesian;

pub use calculator::ConfidenceCalculator;
pub use bayesian::BayesianConfidenceUpdater;
