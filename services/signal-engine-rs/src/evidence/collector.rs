//! Evidence collection system

use crate::evidence::{EvidenceCollection, FactorEvidence, PatternEvidence};
use crate::confluence::factors::{ConfluenceFactor, FactorType};

/// Collects evidence from various signal components
#[derive(Debug)]
pub struct EvidenceCollector {
    collection: EvidenceCollection,
}

impl EvidenceCollector {
    /// Create new evidence collector
    pub fn new() -> Self {
        Self {
            collection: EvidenceCollection {
                patterns: Vec::new(),
                factors: Vec::new(),
                reasons: Vec::new(),
            },
        }
    }

    /// Add pattern evidence
    pub fn add_pattern(&mut self, pattern: PatternEvidence) {
        self.collection.patterns.push(pattern);
    }

    /// Add pattern with simple construction
    pub fn add_pattern_simple(
        &mut self,
        pattern_type: &str,
        location: &str,
        strength: f64,
        age_bars: u32,
    ) {
        self.add_pattern(PatternEvidence {
            pattern_type: pattern_type.to_string(),
            location: location.to_string(),
            strength,
            age_bars,
        });
    }

    /// Add factor evidence
    pub fn add_factor(&mut self, factor: FactorEvidence) {
        self.collection.factors.push(factor);
    }

    /// Add factor from confluence factor
    pub fn add_confluence_factor(&mut self, factor: &ConfluenceFactor) {
        let interpretation = match factor.factor_type {
            FactorType::TimeframeAlignment => {
                format!("HTF alignment: {:.0}% confidence", factor.raw_value * 100.0)
            }
            FactorType::TrendQuality => {
                if factor.contribution > 10.0 {
                    "Strong trend alignment".to_string()
                } else {
                    "Weak or counter-trend".to_string()
                }
            }
            FactorType::Regime => {
                format!("Regime favorability: {:.0}%", factor.raw_value * 100.0)
            }
            FactorType::OrderBlock => {
                if factor.raw_value > 0.5 {
                    "Fresh order block present".to_string()
                } else {
                    "No clear order block".to_string()
                }
            }
            FactorType::FairValueGap => {
                format!("FVG strength: {:.0}%", factor.raw_value * 100.0)
            }
            FactorType::Liquidity => {
                if factor.raw_value > 0.5 {
                    "Liquidity sweep confirmed".to_string()
                } else {
                    "No liquidity sweep".to_string()
                }
            }
            FactorType::Displacement => {
                format!("Displacement: {:.0}%", factor.raw_value * 100.0)
            }
            FactorType::Momentum => {
                format!("Momentum: {:.0}%", factor.raw_value * 100.0)
            }
            FactorType::Volatility => {
                if factor.raw_value < 0.5 {
                    "Favorable volatility".to_string()
                } else {
                    "Elevated volatility".to_string()
                }
            }
            FactorType::Session => {
                format!("Session context: {:.0}%", factor.raw_value * 100.0)
            }
            FactorType::Structure => {
                format!("Structure clarity: {:.0}%", factor.raw_value * 100.0)
            }
            FactorType::RiskReward => {
                format!("R:R = {:.1}:1", factor.raw_value)
            }
        };

        self.add_factor(FactorEvidence {
            factor_type: format!("{:?}", factor.factor_type),
            raw_value: factor.raw_value,
            contribution: factor.contribution,
            interpretation: interpretation.clone(),
        });

        // Also add to reasons
        self.collection.reasons.push(format!(
            "{} (contribution: {:.1})",
            interpretation,
            factor.contribution
        ));
    }

    /// Add a human-readable reason
    pub fn add_reason(&mut self, reason: &str) {
        self.collection.reasons.push(reason.to_string());
    }

    /// Generate "Why Buy" explanation
    pub fn generate_why_buy(&self) -> String {
        let mut parts = Vec::new();

        for pattern in &self.collection.patterns {
            if pattern.strength > 0.6 {
                parts.push(format!(
                    "Strong {} at {} ({:.0}% confidence)",
                    pattern.pattern_type,
                    pattern.location,
                    pattern.strength * 100.0
                ));
            }
        }

        for factor in &self.collection.factors {
            if factor.contribution > 10.0 {
                parts.push(factor.interpretation.clone());
            }
        }

        if parts.is_empty() {
            "Insufficient bullish evidence".to_string()
        } else {
            parts.join("; ")
        }
    }

    /// Generate "Why Sell" explanation
    pub fn generate_why_sell(&self) -> String {
        let mut parts = Vec::new();

        for pattern in &self.collection.patterns {
            pattern.pattern_type.to_lowercase().contains("bear");
            parts.push(format!(
                "{} at {} (strength: {:.0}%)",
                pattern.pattern_type,
                pattern.location,
                pattern.strength * 100.0
            ));
        }

        if parts.is_empty() {
            "Insufficient bearish evidence".to_string()
        } else {
            parts.join("; ")
        }
    }

    /// Generate "Why Now" explanation
    pub fn generate_why_now(&self) -> String {
        // Look for timing-related factors
        let timing_factors: Vec<_> = self.collection.factors.iter()
            .filter(|f| {
                f.factor_type.contains("Timeframe") ||
                f.factor_type.contains("Momentum") ||
                f.factor_type.contains("Displacement")
            })
            .collect();

        if timing_factors.is_empty() {
            "Current conditions align for entry".to_string()
        } else {
            timing_factors.iter()
                .map(|f| f.interpretation.clone())
                .collect::<Vec<_>>()
                .join("; ")
        }
    }

    /// Generate "Why Wait" explanation
    pub fn generate_why_wait(&self) -> String {
        let mut concerns = Vec::new();

        for factor in &self.collection.factors {
            if factor.contribution < 5.0 {
                concerns.push(format!(
                    "Low {} contribution",
                    factor.factor_type
                ));
            }
        }

        if concerns.is_empty() {
            "No significant concerns".to_string()
        } else {
            format!("Wait for: {}", concerns.join("; "))
        }
    }

    /// Generate "What Reduced Confidence" explanation
    pub fn generate_confidence_reducers(&self) -> Vec<String> {
        let mut reducers = Vec::new();

        for factor in &self.collection.factors {
            if factor.contribution < 5.0 {
                reducers.push(format!(
                    "{} factor (value: {:.2}, contribution: {:.1})",
                    factor.factor_type,
                    factor.raw_value,
                    factor.contribution
                ));
            }
        }

        reducers
    }

    /// Generate "What Patterns Contributed" explanation
    pub fn generate_pattern_contributions(&self) -> Vec<String> {
        self.collection.patterns.iter()
            .filter(|p| p.strength > 0.3)
            .map(|p| {
                format!(
                    "{} at {} (strength: {:.0}%, age: {} bars)",
                    p.pattern_type,
                    p.location,
                    p.strength * 100.0,
                    p.age_bars
                )
            })
            .collect()
    }

    /// Finalize and get evidence collection
    pub fn finalize(self) -> EvidenceCollection {
        self.collection
    }

    /// Get current collection (non-consuming)
    pub fn collection(&self) -> &EvidenceCollection {
        &self.collection
    }
}

impl Default for EvidenceCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_pattern() {
        let mut collector = EvidenceCollector::new();
        collector.add_pattern_simple("OrderBlock", "104.50", 0.8, 5);

        assert_eq!(collector.collection().patterns.len(), 1);
    }

    #[test]
    fn test_generate_why_buy() {
        let mut collector = EvidenceCollector::new();
        collector.add_pattern_simple("Bullish OB", "104.50", 0.8, 5);
        collector.add_factor(FactorEvidence {
            factor_type: "Trend".to_string(),
            raw_value: 0.75,
            contribution: 15.0,
            interpretation: "Strong uptrend".to_string(),
        });

        let why_buy = collector.generate_why_buy();
        assert!(!why_buy.is_empty());
    }

    #[test]
    fn test_generate_pattern_contributions() {
        let mut collector = EvidenceCollector::new();
        collector.add_pattern_simple("FVG", "100.00", 0.7, 3);
        collector.add_pattern_simple("Sweep", "99.50", 0.4, 10);

        let contributions = collector.generate_pattern_contributions();
        assert_eq!(contributions.len(), 1); // Only the FVG (strength > 0.3)
    }
}
