//! Confluence scoring engine

pub mod engine;
pub mod factors;
pub mod weights;

pub use engine::ConfluenceEngine;
pub use factors::{ConfluenceFactor, FactorBuilder, FactorType};
pub use weights::{AdaptiveWeights, WeightAdjuster};

/// Confluence score (0-100)
#[derive(Debug, Clone)]
pub struct ConfluenceScore {
    /// Total score
    pub total: u8,
    /// Individual factor contributions
    pub factors: Vec<ConfluenceFactor>,
}

impl ConfluenceScore {
    /// Create new empty score
    pub fn empty() -> Self {
        Self {
            total: 0,
            factors: vec![],
        }
    }

    /// Check if score meets minimum threshold
    pub fn meets_threshold(&self, threshold: u8) -> bool {
        self.total >= threshold
    }

    /// Get score as grade
    pub fn as_grade(&self) -> crate::config::SignalQuality {
        match self.total {
            85..=100 => crate::config::SignalQuality::APlus,
            70..=84 => crate::config::SignalQuality::A,
            60..=69 => crate::config::SignalQuality::B,
            _ => crate::config::SignalQuality::Reject,
        }
    }

    /// Get factor contribution by type
    pub fn factor_contribution(&self, factor_type: FactorType) -> Option<f64> {
        self.factors
            .iter()
            .find(|f| f.factor_type == factor_type)
            .map(|f| f.contribution)
    }
}

impl Default for ConfluenceScore {
    fn default() -> Self {
        Self::empty()
    }
}

/// Confluence analysis result
#[derive(Debug, Clone)]
pub struct ConfluenceAnalysis {
    /// Overall confluence score
    pub score: ConfluenceScore,
    /// Signal quality grade
    pub quality: crate::config::SignalQuality,
    /// Primary contributing factors
    pub primary_factors: Vec<String>,
    /// Concerns/red flags
    pub concerns: Vec<String>,
}

impl ConfluenceAnalysis {
    /// Check if analysis suggests trading
    pub fn is_tradable(&self) -> bool {
        self.quality != crate::config::SignalQuality::Reject
    }
}
