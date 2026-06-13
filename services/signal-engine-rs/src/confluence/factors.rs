//! Confluence factors for signal quality

use serde::{Deserialize, Serialize};

/// Individual confluence factor contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfluenceFactor {
    /// Type of factor
    pub factor_type: FactorType,
    /// Human-readable name
    pub name: String,
    /// Raw value
    pub raw_value: f64,
    /// Weight applied
    pub weight: f64,
    /// Score contribution (0-100)
    pub contribution: f64,
    /// Description
    pub description: String,
}

/// Types of confluence factors
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FactorType {
    /// Multi-timeframe alignment
    TimeframeAlignment,
    /// Trend quality
    TrendQuality,
    /// Market regime fit
    Regime,
    /// Order block presence
    OrderBlock,
    /// FVG alignment
    FairValueGap,
    /// Liquidity sweep
    Liquidity,
    /// Displacement/impulse
    Displacement,
    /// Momentum
    Momentum,
    /// Volatility condition
    Volatility,
    /// Session/timing
    Session,
    /// Structure quality
    Structure,
    /// Risk/reward ratio
    RiskReward,
}

impl FactorType {
    /// Get default weight for factor type
    pub fn default_weight(&self) -> f64 {
        match self {
            FactorType::TimeframeAlignment => 1.2,    // Strong - HTF context
            FactorType::TrendQuality => 1.0,           // Standard
            FactorType::Regime => 0.8,                 // Moderate
            FactorType::OrderBlock => 1.0,             // Standard
            FactorType::FairValueGap => 0.7,           // Secondary
            FactorType::Liquidity => 0.9,              // Important
            FactorType::Displacement => 1.1,           // Strong signal
            FactorType::Momentum => 0.8,               // Moderate
            FactorType::Volatility => 0.6,             // Lower weight
            FactorType::Session => 0.5,                // Lower weight
            FactorType::Structure => 1.0,              // Standard
            FactorType::RiskReward => 1.0,             // Hard filter
        }
    }

    /// Maximum possible score for this factor
    pub fn max_score(&self) -> f64 {
        20.0 // Each factor can contribute up to 20 points weighted
    }
}

/// Factor contribution builder
#[derive(Debug, Default)]
pub struct FactorBuilder {
    factors: Vec<ConfluenceFactor>,
}

impl FactorBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add factor
    pub fn add(&mut self, factor: ConfluenceFactor) {
        self.factors.push(factor);
    }

    /// Add factor with simple construction
    pub fn add_simple(
        &mut self,
        factor_type: FactorType,
        name: &str,
        raw_value: f64,
        weight: f64,
        contribution: f64,
        description: &str,
    ) {
        self.factors.push(ConfluenceFactor {
            factor_type,
            name: name.to_string(),
            raw_value,
            weight,
            contribution: contribution.max(0.0).min(20.0), // Cap at max Score
            description: description.to_string(),
        });
    }

    /// Add timeframe alignment factor
    pub fn add_timeframe_alignment(&mut self, alignment_score: f64) {
        let (contribution, desc) = match alignment_score {
            s if s >= 0.9 => (18.0, "Excellent HTF alignment".to_string()),
            s if s >= 0.7 => (14.0, "Good HTF alignment".to_string()),
            s if s >= 0.5 => (10.0, "Moderate HTF alignment".to_string()),
            s if s >= 0.3 => (6.0, "Weak HTF alignment".to_string()),
            _ => (0.0, "No HTF alignment".to_string()),
        };

        self.add_simple(
            FactorType::TimeframeAlignment,
            "Timeframe Alignment",
            alignment_score,
            FactorType::TimeframeAlignment.default_weight(),
            contribution,
            &desc,
        );
    }

    /// Add trend quality factor
    pub fn add_trend_quality(&mut self, trend_strength: f64, trend_aligned: bool) {
        let base_contribution = match trend_strength {
            s if s >= 0.8 => 16.0,
            s if s >= 0.6 => 12.0,
            s if s >= 0.4 => 8.0,
            _ => 4.0,
        };

        let contribution = if trend_aligned {
            base_contribution
        } else {
            base_contribution * 0.3 // Reduced if against trend
        };

        let desc = if trend_aligned {
            format!("Strong trend alignment ({:.0}%)", trend_strength * 100.0)
        } else {
            format!("Counter-trend ({}%)", trend_strength * 100.0)
        };

        self.add_simple(
            FactorType::TrendQuality,
            "Trend Quality",
            trend_strength,
            FactorType::TrendQuality.default_weight(),
            contribution,
            &desc,
        );
    }

    /// Add regime factor
    pub fn add_regime(&mut self, regime_score: f64, regime_favorable: bool) {
        let contribution = if regime_favorable {
            match regime_score {
                s if s >= 0.8 => 14.0,
                s if s >= 0.6 => 10.0,
                s if s >= 0.4 => 6.0,
                _ => 3.0,
            }
        } else {
            regime_score * 5.0 // Reduced in unfavorable regime
        };

        let desc = if regime_favorable {
            "Favorable market regime".to_string()
        } else {
            "Challenging regime conditions".to_string()
        };

        self.add_simple(
            FactorType::Regime,
            "Market Regime",
            regime_score,
            FactorType::Regime.default_weight(),
            contribution,
            &desc,
        );
    }

    /// Add order block factor
    pub fn add_order_block(&mut self, ob_strength: f64, ob_aligned: bool, fresh: bool) {
        let base_contribution = match ob_strength {
            s if s >= 0.8 => 16.0,
            s if s >= 0.6 => 12.0,
            s if s >= 0.4 => 8.0,
            _ => 4.0,
        };

        let freshness_multiplier = if fresh { 1.0 } else { 0.5 };
        let contribution = base_contribution * freshness_multiplier;

        let desc = if ob_aligned {
            if fresh {
                format!("Fresh OB aligned ({:.0}%)", ob_strength * 100.0)
            } else {
                format!("Mitigated OB aligned ({:.0}%)", ob_strength * 100.0)
            }
        } else {
            "OB structure not aligned".to_string()
        };

        self.add_simple(
            FactorType::OrderBlock,
            "Order Block",
            ob_strength,
            FactorType::OrderBlock.default_weight(),
            contribution,
            &desc,
        );
    }

    /// Add FVG factor
    pub fn add_fvg(&mut self, fvg_strength: f64, fvg_aligned: bool) {
        let base_contribution = match fvg_strength {
            s if s >= 0.7 => 12.0,
            s if s >= 0.5 => 8.0,
            s if s >= 0.3 => 4.0,
            _ => 2.0,
        };

        let contribution = if fvg_aligned { base_contribution } else { base_contribution * 0.3 };

        let desc = if fvg_aligned {
            format!("FVG aligned ({:.0}%)", fvg_strength * 100.0)
        } else {
            "No relevant FVG".to_string()
        };

        self.add_simple(
            FactorType::FairValueGap,
            "Fair Value Gap",
            fvg_strength,
            FactorType::FairValueGap.default_weight(),
            contribution,
            &desc,
        );
    }

    /// Add liquidity sweep factor
    pub fn add_liquidity(&mut self, sweep_strength: f64, sweep_aligned: bool) {
        let base_contribution = match sweep_strength {
            s if s >= 0.8 => 16.0,
            s if s >= 0.6 => 12.0,
            s if s >= 0.4 => 8.0,
            _ => 4.0,
        };

        let contribution = if sweep_aligned { base_contribution } else { 0.0 };

        let desc = if sweep_aligned {
            format!("Liquidity sweep confirmed ({:.0}%)", sweep_strength * 100.0)
        } else {
            "No recent sweep".to_string()
        };

        self.add_simple(
            FactorType::Liquidity,
            "Liquidity",
            sweep_strength,
            FactorType::Liquidity.default_weight(),
            contribution,
            &desc,
        );
    }

    /// Add displacement factor
    pub fn add_displacement(&mut self, displacement_strength: f64, direction_aligned: bool) {
        let base_contribution = match displacement_strength {
            s if s >= 0.7 => 16.0,
            s if s >= 0.5 => 11.0,
            s if s >= 0.3 => 6.0,
            _ => 2.0,
        };

        let contribution = if direction_aligned { base_contribution } else { base_contribution * 0.2 };

        let desc = if direction_aligned {
            format!("Displacement aligned ({:.0}%)", displacement_strength * 100.0)
        } else {
            "Displacement opposing".to_string()
        };

        self.add_simple(
            FactorType::Displacement,
            "Displacement",
            displacement_strength,
            FactorType::Displacement.default_weight(),
            contribution,
            &desc,
        );
    }

    /// Add momentum factor
    pub fn add_momentum(&mut self, momentum_score: f64, direction_aligned: bool) {
        let base_contribution = match momentum_score {
            s if s >= 0.7 => 12.0,
            s if s >= 0.5 => 9.0,
            s if s >= 0.3 => 5.0,
            _ => 2.0,
        };

        let contribution = if direction_aligned { base_contribution } else { base_contribution * 0.2 };

        self.add_simple(
            FactorType::Momentum,
            "Momentum",
            momentum_score,
            FactorType::Momentum.default_weight(),
            contribution,
            &format!("Momentum {:.0}%", momentum_score * 100.0),
        );
    }

    /// Add volatility factor
    pub fn add_volatility(&mut self, volatility_score: f64, favorable: bool) {
        let contribution = if favorable {
            match volatility_score {
                s if s >= 0.7 => 10.0,
                s if s >= 0.4 => 7.0,
                _ => 4.0,
            }
        } else {
            0.0 // No contribution in unfavorable volatility
        };

        self.add_simple(
            FactorType::Volatility,
            "Volatility",
            volatility_score,
            FactorType::Volatility.default_weight(),
            contribution,
            &format!("Volatility {:.0}%", volatility_score * 100.0),
        );
    }

    /// Add structure factor
    pub fn add_structure(&mut self, structure_quality: f64, clear_swings: bool) {
        let base_contribution = match structure_quality {
            s if s >= 0.8 => 16.0,
            s if s >= 0.6 => 12.0,
            s if s >= 0.4 => 8.0,
            _ => 4.0,
        };

        let contribution = if clear_swings { base_contribution } else { base_contribution * 0.5 };

        let desc = if clear_swings {
            format!("Clear structure ({:.0}%)", structure_quality * 100.0)
        } else {
            format!("Unclear structure ({:.0}%)", structure_quality * 100.0)
        };

        self.add_simple(
            FactorType::Structure,
            "Market Structure",
            structure_quality,
            FactorType::Structure.default_weight(),
            contribution,
            &desc,
        );
    }

    /// Add risk/reward factor
    pub fn add_risk_reward(&mut self, rr_ratio: f64) {
        let contribution = match rr_ratio {
            r if r >= 4.0 => 20.0,
            r if r >= 3.0 => 18.0,
            r if r >= 2.5 => 16.0,
            r if r >= 2.0 => 14.0,
            r if r >= 1.5 => 10.0,
            r if r >= 1.0 => 5.0,
            _ => 0.0,
        };

        self.add_simple(
            FactorType::RiskReward,
            "Risk/Reward",
            rr_ratio,
            FactorType::RiskReward.default_weight(),
            contribution,
            &format!("R:R {:.1}:1", rr_ratio),
        );
    }

    /// Build and return factors
    pub fn build(self) -> Vec<ConfluenceFactor> {
        self.factors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factor_builder() {
        let mut builder = FactorBuilder::new();

        builder.add_timeframe_alignment(0.85);
        builder.add_trend_quality(0.75, true);
        builder.add_risk_reward(2.5);

        let factors = builder.build();

        assert_eq!(factors.len(), 3);
        assert!(factors.iter().any(|f| f.factor_type == FactorType::TimeframeAlignment));
        assert!(factors.iter().any(|f| f.factor_type == FactorType::TrendQuality));
    }

    #[test]
    fn test_risk_reward_scoring() {
        let mut builder = FactorBuilder::new();
        builder.add_risk_reward(3.0);
        let factors = builder.build();

        assert_eq!(factors[0].contribution, 18.0);
    }
}
