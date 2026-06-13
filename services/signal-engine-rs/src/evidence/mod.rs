//! Explainability and evidence collection

pub mod collector;

pub use collector::EvidenceCollector;

/// Evidence collection for a signal
#[derive(Debug, Clone)]
pub struct EvidenceCollection {
    /// Pattern evidence
    pub patterns: Vec<PatternEvidence>,
    /// Factor contributions
    pub factors: Vec<FactorEvidence>,
    /// Human readable reasons
    pub reasons: Vec<String>,
}

impl EvidenceCollection {
    /// Create empty evidence collection
    pub fn empty() -> Self {
        Self {
            patterns: vec![],
            factors: vec![],
            reasons: vec![],
        }
    }

    /// Check if collection has evidence
    pub fn has_evidence(&self) -> bool {
        !self.patterns.is_empty() || !self.factors.is_empty()
    }

    /// Get pattern count
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Get strongest pattern
    pub fn strongest_pattern(&self) -> Option<&PatternEvidence> {
        self.patterns.iter().max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap())
    }
}

/// Evidence for a detected pattern
#[derive(Debug, Clone)]
pub struct PatternEvidence {
    /// Pattern type
    pub pattern_type: String,
    /// Pattern location
    pub location: String,
    /// Pattern strength (0.0 - 1.0)
    pub strength: f64,
    /// Pattern age in bars
    pub age_bars: u32,
}

/// Evidence for a confluence factor
#[derive(Debug, Clone)]
pub struct FactorEvidence {
    /// Factor type
    pub factor_type: String,
    /// Raw value
    pub raw_value: f64,
    /// Score contribution
    pub contribution: f64,
    /// Interpretation
    pub interpretation: String,
}
