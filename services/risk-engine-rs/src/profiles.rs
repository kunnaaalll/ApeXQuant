//! Risk profiles for dynamic capital allocation
use rust_decimal::prelude::FromPrimitive;

use crate::{ConfidenceScore, MarketSession};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Risk profile determines position sizing and risk acceptance
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RiskProfile {
    /// Ultra-conservative - minimal risk, maximum preservation
    VeryConservative,
    /// Conservative - reduced risk during uncertain periods
    Conservative,
    /// Normal - standard risk parameters
    Normal,
    /// Aggressive - increased risk for strong opportunities
    Aggressive,
    /// High conviction - maximum allocation for exceptional setups
    HighConviction,
}

impl RiskProfile {
    /// Get the base risk multiplier for this profile
    pub fn risk_multiplier(&self) -> Decimal {
        match self {
            RiskProfile::VeryConservative => Decimal::from_f64(0.5).unwrap_or(Decimal::ONE),
            RiskProfile::Conservative => Decimal::from_f64(0.75).unwrap_or(Decimal::ONE),
            RiskProfile::Normal => Decimal::ONE,
            RiskProfile::Aggressive => Decimal::from_f64(1.25).unwrap_or(Decimal::ONE),
            RiskProfile::HighConviction => Decimal::from_f64(1.5).unwrap_or(Decimal::ONE),
        }
    }

    /// Get maximum position count for this profile
    pub fn max_positions(&self) -> u8 {
        match self {
            RiskProfile::VeryConservative => 2,
            RiskProfile::Conservative => 3,
            RiskProfile::Normal => 5,
            RiskProfile::Aggressive => 6,
            RiskProfile::HighConviction => 4, // Fewer, larger positions
        }
    }

    /// Get maximum correlation allowed for this profile
    pub fn max_correlation(&self) -> Decimal {
        match self {
            RiskProfile::VeryConservative => Decimal::from_f64(0.3).unwrap_or(Decimal::ZERO),
            RiskProfile::Conservative => Decimal::from_f64(0.5).unwrap_or(Decimal::ZERO),
            RiskProfile::Normal => Decimal::from_f64(0.7).unwrap_or(Decimal::ZERO),
            RiskProfile::Aggressive => Decimal::from_f64(0.85).unwrap_or(Decimal::ZERO),
            RiskProfile::HighConviction => Decimal::from_f64(0.8).unwrap_or(Decimal::ZERO),
        }
    }

    /// Get the minimum confluence score required for this profile
    pub fn min_confluence(&self) -> Decimal {
        match self {
            RiskProfile::VeryConservative => Decimal::from(8),
            RiskProfile::Conservative => Decimal::from(6),
            RiskProfile::Normal => Decimal::from(5),
            RiskProfile::Aggressive => Decimal::from(4),
            RiskProfile::HighConviction => Decimal::from(9),
        }
    }

    /// Whether this profile allows martingale-style doubling down
    pub fn allows_martingale(&self) -> bool {
        matches!(self, RiskProfile::HighConviction | RiskProfile::Aggressive)
    }

    /// Get position sizing priority (higher = larger individual positions)
    pub fn position_sizing_priority(&self) -> Decimal {
        match self {
            RiskProfile::VeryConservative => Decimal::from_f64(0.5).unwrap_or(Decimal::ONE),
            RiskProfile::Conservative => Decimal::from_f64(0.75).unwrap_or(Decimal::ONE),
            RiskProfile::Normal => Decimal::ONE,
            RiskProfile::Aggressive => Decimal::from_f64(1.25).unwrap_or(Decimal::ONE),
            RiskProfile::HighConviction => Decimal::from_f64(2.0).unwrap_or(Decimal::ONE),
        }
    }
}

impl fmt::Display for RiskProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            RiskProfile::VeryConservative => "very_conservative",
            RiskProfile::Conservative => "conservative",
            RiskProfile::Normal => "normal",
            RiskProfile::Aggressive => "aggressive",
            RiskProfile::HighConviction => "high_conviction",
        })
    }
}

impl Default for RiskProfile {
    fn default() -> Self {
        RiskProfile::Normal
    }
}

/// Engine for selecting appropriate risk profiles
pub struct RiskProfileEngine;

impl RiskProfileEngine {
    /// Create new risk profile engine
    pub fn new() -> Self {
        Self
    }

    /// Select risk profile based on conditions
    pub fn select(&self, confidence: &ConfidenceScore, session: &MarketSession) -> RiskProfile {
        // Start with confidence-based selection
        let base_profile = self.profile_from_confidence(confidence.overall);

        // Adjust for session quality
        self.adjust_for_session(base_profile, session, confidence.market_condition_quality)
    }

    /// Determine profile from confidence score (0-1)
    fn profile_from_confidence(&self, confidence: Decimal) -> RiskProfile {
        if confidence >= Decimal::from_f64(0.95).unwrap_or(Decimal::ONE) {
            RiskProfile::HighConviction
        } else if confidence >= Decimal::from_f64(0.85).unwrap_or(Decimal::ONE) {
            RiskProfile::Aggressive
        } else if confidence >= Decimal::from_f64(0.70).unwrap_or(Decimal::ONE) {
            RiskProfile::Normal
        } else if confidence >= Decimal::from_f64(0.50).unwrap_or(Decimal::ONE) {
            RiskProfile::Conservative
        } else {
            RiskProfile::VeryConservative
        }
    }

    /// Adjust profile based on session conditions
    fn adjust_for_session(
        &self,
        current: RiskProfile,
        session: &MarketSession,
        market_condition_quality: Decimal,
    ) -> RiskProfile {
        // Poor market conditions push toward conservative
        if market_condition_quality < Decimal::from_f64(0.4).unwrap_or(Decimal::ZERO) {
            return match current {
                RiskProfile::HighConviction => RiskProfile::Aggressive,
                RiskProfile::Aggressive => RiskProfile::Normal,
                RiskProfile::Normal => RiskProfile::Conservative,
                RiskProfile::Conservative | RiskProfile::VeryConservative => RiskProfile::VeryConservative,
            };
        }

        // Session adjustments
        match session {
            MarketSession::OverlapLondonNy => {
                // Best liquidity - can be more aggressive if conditions support
                if current == RiskProfile::Normal && market_condition_quality > Decimal::from_f64(0.8).unwrap_or(Decimal::ONE) {
                    RiskProfile::Aggressive
                } else {
                    current
                }
            }
            MarketSession::Asia => {
                // Lower liquidity - be more conservative unless high conviction
                match current {
                    RiskProfile::Aggressive => RiskProfile::Normal,
                    RiskProfile::HighConviction => RiskProfile::Aggressive,
                    _ => current,
                }
            }
            _ => current,
        }
    }

    /// Force a specific profile (for override scenarios)
    pub fn force_profile(&self, profile: RiskProfile) -> RiskProfile {
        profile
    }

    /// Get the next more conservative profile
    pub fn step_down(&self, current: RiskProfile) -> RiskProfile {
        match current {
            RiskProfile::HighConviction => RiskProfile::Aggressive,
            RiskProfile::Aggressive => RiskProfile::Normal,
            RiskProfile::Normal => RiskProfile::Conservative,
            RiskProfile::Conservative | RiskProfile::VeryConservative => RiskProfile::VeryConservative,
        }
    }

    /// Get the next more aggressive profile
    pub fn step_up(&self, current: RiskProfile) -> RiskProfile {
        match current {
            RiskProfile::VeryConservative => RiskProfile::Conservative,
            RiskProfile::Conservative => RiskProfile::Normal,
            RiskProfile::Normal => RiskProfile::Aggressive,
            RiskProfile::Aggressive | RiskProfile::HighConviction => RiskProfile::HighConviction,
        }
    }
}

impl Default for RiskProfileEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for objects that can provide risk profile selection
pub trait RiskProfileSelector {
    /// Select the appropriate risk profile
    fn select_profile(&self, confidence: &ConfidenceScore, session: &MarketSession) -> RiskProfile;
}

impl RiskProfileSelector for RiskProfileEngine {
    fn select_profile(&self, confidence: &ConfidenceScore, session: &MarketSession) -> RiskProfile {
        self.select(confidence, session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_risk_multipliers() {
        assert_eq!(RiskProfile::VeryConservative.risk_multiplier(), Decimal::from_f64(0.5).unwrap());
        assert_eq!(RiskProfile::Normal.risk_multiplier(), Decimal::ONE);
        assert_eq!(RiskProfile::HighConviction.risk_multiplier(), Decimal::from_f64(1.5).unwrap());
    }

    #[test]
    fn test_profile_selection() {
        let engine = RiskProfileEngine::new();

        let high_confidence = ConfidenceScore {
            overall: Decimal::from_f64(0.95).unwrap(),
            signal: Decimal::from_f64(0.9).unwrap(),
            confluence: Decimal::from_f64(0.95).unwrap(),
            regime: Decimal::from_f64(0.9).unwrap(),
            market_condition_quality: Decimal::from_f64(0.85).unwrap(),
        };

        let profile = engine.select(&high_confidence, &MarketSession::London);
        assert_eq!(profile, RiskProfile::HighConviction);
    }

    #[test]
    fn test_step_up_down() {
        let engine = RiskProfileEngine::new();

        assert_eq!(engine.step_down(RiskProfile::Aggressive), RiskProfile::Normal);
        assert_eq!(engine.step_up(RiskProfile::Conservative), RiskProfile::Normal);
    }

    #[test]
    fn test_max_positions() {
        assert_eq!(RiskProfile::VeryConservative.max_positions(), 2);
        assert_eq!(RiskProfile::HighConviction.max_positions(), 4);
    }

    #[test]
    fn test_high_conviction_min_confluence() {
        assert_eq!(RiskProfile::HighConviction.min_confluence(), Decimal::from(9));
    }
}
