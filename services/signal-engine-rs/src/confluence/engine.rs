//! Confluence scoring engine
use num_traits::ToPrimitive;

use crate::confluence::factors::{ConfluenceFactor, FactorBuilder};
use crate::confluence::ConfluenceScore;
use crate::market_data::Candle;
use crate::mtf::types::{MTFAlignmentResult, MarketBias};
use crate::regime::MarketRegime;
use crate::signals::result::SignalDirection;
use crate::smc::SMCAnalysis;
use crate::structure::StructureAnalysis;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Engine for calculating confluence scores
#[derive(Debug)]
pub struct ConfluenceEngine {
    min_score: u8,
}

impl ConfluenceEngine {
    /// Create new confluence engine
    pub fn new(min_score: u8) -> Self {
        Self { min_score }
    }

    /// Calculate confluence score for a potential signal
    pub fn calculate_score(
        &self,
        direction: SignalDirection,
        candles: &HashMap<String, Vec<Candle>>,
        structure: &StructureAnalysis,
        mtf_alignment: &MTFAlignmentResult,
        regime: &MarketRegime,
        smc: &SMCAnalysis,
        entry_price: Decimal,
        stop_price: Decimal,
        target_price: Decimal,
    ) -> ConfluenceScore {
        let mut builder = FactorBuilder::new();

        // 1. Timeframe alignment factor
        let tf_score = self.calculate_timeframe_factor(mtf_alignment, direction);
        builder.add_timeframe_alignment(tf_score);

        // 2. Trend quality factor
        let trend_score = self.calculate_trend_factor(structure, direction);
        builder.add_trend_quality(trend_score.0, trend_score.1);

        // 3. Regime factor
        let regime_result = self.calculate_regime_factor(regime, direction);
        builder.add_regime(regime_result.0, regime_result.1);

        // 4. Structure quality
        let structure_score = self.calculate_structure_factor(structure);
        builder.add_structure(structure_score.0, structure_score.1);

        // 5. Order block factor
        let (ob_strength, ob_aligned, ob_fresh) = self.calculate_order_block_factor(smc, direction);
        builder.add_order_block(ob_strength, ob_aligned, ob_fresh);

        // 6. FVG factor
        let (fvg_strength, fvg_aligned) = self.calculate_fvg_factor(smc, direction, entry_price);
        builder.add_fvg(fvg_strength, fvg_aligned);

        // 7. Liquidity factor
        let (liq_strength, liq_aligned) = self.calculate_liquidity_factor(smc, direction);
        builder.add_liquidity(liq_strength, liq_aligned);

        // 8. Displacement factor
        let (disp_strength, disp_aligned) = self.calculate_displacement_factor(smc, direction);
        builder.add_displacement(disp_strength, disp_aligned);

        // 9. Momentum factor
        let (mom_score, mom_aligned) = self.calculate_momentum_factor(candles, direction);
        builder.add_momentum(mom_score, mom_aligned);

        // 10. Volatility factor
        let (vol_score, vol_favorable) = self.calculate_volatility_factor(regime);
        builder.add_volatility(vol_score, vol_favorable);

        // 11. Risk/Reward factor
        let rr = calculate_risk_reward(entry_price, stop_price, target_price);
        builder.add_risk_reward(rr);

        let factors = builder.build();
        let total = calculate_weighted_total(&factors);

        ConfluenceScore {
            total: total.min(100.0) as u8,
            factors,
        }
    }

    /// Check if score meets threshold
    pub fn meets_threshold(&self, score: &ConfluenceScore) -> bool {
        score.total >= self.min_score
    }

    // Private calculation methods

    fn calculate_timeframe_factor(
        &self,
        mtf: &MTFAlignmentResult,
        direction: SignalDirection,
    ) -> f64 {
        // Check alignment with directional bias
        let aligned = match (mtf.bias, &direction) {
            (MarketBias::StrongBullish, SignalDirection::Long) => true,
            (MarketBias::Bullish, SignalDirection::Long) => true,
            (MarketBias::StrongBearish, SignalDirection::Short) => true,
            (MarketBias::Bearish, SignalDirection::Short) => true,
            _ => false,
        };

        if aligned {
            mtf.alignment_score
        } else {
            mtf.alignment_score * 0.3
        }
    }

    fn calculate_trend_factor(
        &self,
        structure: &StructureAnalysis,
        direction: SignalDirection,
    ) -> (f64, bool) {
        use crate::structure::trend::TrendDirection;

        let trend_aligned = match (structure.trend, &direction) {
            (TrendDirection::Up, SignalDirection::Long) => true,
            (TrendDirection::Down, SignalDirection::Short) => true,
            _ => false,
        };

        // Trend strength based on swing clarity
        let swing_count = structure.swing_highs.len().min(structure.swing_lows.len());
        let trend_strength = match swing_count {
            0 => 0.0,
            1 => 0.3,
            2 => 0.5,
            3 => 0.7,
            _ => 0.85,
        };

        (trend_strength, trend_aligned)
    }

    fn calculate_regime_factor(
        &self,
        regime: &MarketRegime,
        direction: SignalDirection,
    ) -> (f64, bool) {
        use crate::regime::RegimeType;

        let favorable = match regime.regime_type {
            RegimeType::TrendingUp => matches!(direction, SignalDirection::Long),
            RegimeType::TrendingDown => matches!(direction, SignalDirection::Short),
            RegimeType::Ranging => true,         // Mean reversion works
            RegimeType::HighVolatility => false, // Avoid
            RegimeType::LowVolatility => true,   // Breakout potential
            RegimeType::Transition => false,
            RegimeType::Breakout => true,
            RegimeType::Undefined => false,
        };

        (regime.confidence, favorable)
    }

    fn calculate_structure_factor(&self, structure: &StructureAnalysis) -> (f64, bool) {
        let swing_count = structure.swing_highs.len().min(structure.swing_lows.len());
        let clear_structure = swing_count >= 2;

        let quality = match swing_count {
            0 => 0.1,
            1 => 0.3,
            2 => 0.5,
            3 => 0.7,
            n if n >= 4 => 0.85,
            _ => 0.0,
        };

        (quality, clear_structure)
    }

    fn calculate_order_block_factor(
        &self,
        smc: &SMCAnalysis,
        direction: SignalDirection,
    ) -> (f64, bool, bool) {
        let (ob, aligned, fresh) = match direction {
            SignalDirection::Long => {
                let ob = smc.freshest_bullish_ob();
                let aligned = ob.is_some();
                let fresh = ob.map(|o| !o.mitigated).unwrap_or(false);
                let strength = ob.map(|o| o.strength).unwrap_or(0.0);
                (strength, aligned, fresh)
            }
            SignalDirection::Short => {
                let ob = smc.freshest_bearish_ob();
                let aligned = ob.is_some();
                let fresh = ob.map(|o| !o.mitigated).unwrap_or(false);
                let strength = ob.map(|o| o.strength).unwrap_or(0.0);
                (strength, aligned, fresh)
            }
            SignalDirection::Neutral => (0.0, false, false),
        };

        (ob, aligned, fresh)
    }

    fn calculate_fvg_factor(
        &self,
        smc: &SMCAnalysis,
        direction: SignalDirection,
        entry: Decimal,
    ) -> (f64, bool) {
        use crate::smc::fvg::FVGDirection;

        let fresh_fvgs = smc.fresh_fvgs(30);

        let relevant_fvg = fresh_fvgs.iter().find(|f| match direction {
            SignalDirection::Long => matches!(f.direction, FVGDirection::Bullish),
            SignalDirection::Short => matches!(f.direction, FVGDirection::Bearish),
            SignalDirection::Neutral => false,
        });

        let strength = relevant_fvg.map(|f| f.strength).unwrap_or(0.0);
        let aligned = relevant_fvg.is_some();

        (strength, aligned)
    }

    fn calculate_liquidity_factor(
        &self,
        smc: &SMCAnalysis,
        direction: SignalDirection,
    ) -> (f64, bool) {
        use crate::smc::liquidity::SweepDirection;

        let aligned_sweep = smc
            .sweeps
            .iter()
            .filter(|s| match direction {
                SignalDirection::Long => matches!(s.direction, SweepDirection::Low),
                SignalDirection::Short => matches!(s.direction, SweepDirection::High),
                SignalDirection::Neutral => false,
            })
            .max_by(|a, b| {
                a.strength
                    .partial_cmp(&b.strength)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        let strength = aligned_sweep.map(|s| s.strength).unwrap_or(0.0);
        let aligned = aligned_sweep.is_some();

        (strength, aligned)
    }

    fn calculate_displacement_factor(
        &self,
        smc: &SMCAnalysis,
        direction: SignalDirection,
    ) -> (f64, bool) {
        use crate::smc::displacement::DisplacementDirection;

        let recent_disp = smc.displacements.last();

        let aligned = recent_disp
            .map(|d| match (d.direction, &direction) {
                (DisplacementDirection::Up, SignalDirection::Long) => true,
                (DisplacementDirection::Down, SignalDirection::Short) => true,
                _ => false,
            })
            .unwrap_or(false);

        let strength = recent_disp.map(|d| d.strength).unwrap_or(0.0);

        (strength, aligned)
    }

    fn calculate_momentum_factor(
        &self,
        candles: &HashMap<String, Vec<Candle>>,
        direction: SignalDirection,
    ) -> (f64, bool) {
        let execution_tf = "M15";
        let tf_candles = candles.get(execution_tf);

        let c = match tf_candles {
            Some(candles) if candles.len() >= 10 => candles,
            _ => return (0.0, false),
        };
        let recent = &c[c.len().saturating_sub(10)..];

        // Simple momentum: count bullish/bearish candles
        let bullish = recent.iter().filter(|c| c.close > c.open).count() as f64;
        let bearish = recent.iter().filter(|c| c.close < c.open).count() as f64;
        let total = (bullish + bearish).max(1.0);

        let bullish_ratio = bullish / total;
        let bearish_ratio = bearish / total;

        let (score, aligned) = match direction {
            SignalDirection::Long => (bullish_ratio, bullish_ratio > 0.6),
            SignalDirection::Short => (bearish_ratio, bearish_ratio > 0.6),
            SignalDirection::Neutral => (0.0, false),
        };

        (score, aligned)
    }

    fn calculate_volatility_factor(&self, regime: &MarketRegime) -> (f64, bool) {
        let favorable = !regime.regime_type.requires_caution();
        let score = 1.0 - regime.volatility_percentile;

        (score, favorable)
    }
}

impl Default for ConfluenceEngine {
    fn default() -> Self {
        Self::new(60) // Default 60 minimum score
    }
}

/// Calculate weighted total from factors
fn calculate_weighted_total(factors: &[ConfluenceFactor]) -> f64 {
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for factor in factors {
        weighted_sum += factor.contribution * factor.weight;
        total_weight += factor.weight;
    }

    if total_weight == 0.0 {
        0.0
    } else {
        (weighted_sum / total_weight * factors.len() as f64)
            .min(100.0)
            .max(0.0)
    }
}

/// Calculate risk/reward ratio
fn calculate_risk_reward(entry: Decimal, stop: Decimal, target: Decimal) -> f64 {
    let risk = (entry - stop).abs();
    let reward = (target - entry).abs();

    if risk == Decimal::ZERO {
        0.0
    } else {
        (reward / risk).to_f64().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_reward_calculation() {
        let entry = Decimal::new(10000, 2);
        let stop = Decimal::new(9800, 2);
        let target = Decimal::new(10400, 2);

        let rr = calculate_risk_reward(entry, stop, target);
        assert!((rr - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_weighted_total() {
        let factors = vec![
            ConfluenceFactor {
                factor_type: crate::confluence::factors::FactorType::TrendQuality,
                name: "Trend".to_string(),
                raw_value: 0.8,
                weight: 1.0,
                contribution: 15.0,
                description: "Strong trend".to_string(),
            },
            ConfluenceFactor {
                factor_type: crate::confluence::factors::FactorType::OrderBlock,
                name: "OB".to_string(),
                raw_value: 0.7,
                weight: 1.0,
                contribution: 12.0,
                description: "Good OB".to_string(),
            },
        ];

        let total = calculate_weighted_total(&factors);
        assert!(total > 0.0 && total <= 100.0);
    }
}
