//! Confidence calculation

pub mod bayesian;
pub mod calculator;
pub mod decay;
pub mod factors;

pub use bayesian::BayesianConfidenceUpdater;
pub use calculator::ConfidenceCalculator;
