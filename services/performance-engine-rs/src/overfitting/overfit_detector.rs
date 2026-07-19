use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverfitState {
    Healthy,
    Caution,
    Warning,
    Overfit,
    Dangerous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverfitInput {
    /// Number of distinct parameter variants tested before arriving at current config
    pub parameters_tested: u32,
    /// Number of trades used in optimisation (in-sample)
    pub in_sample_trades: u32,
    /// Number of trades used in out-of-sample validation
    pub out_of_sample_trades: u32,
    /// In-sample expectancy
    pub in_sample_expectancy: Decimal,
    /// Out-of-sample expectancy
    pub out_of_sample_expectancy: Decimal,
    /// Profit factor in-sample
    pub in_sample_pf: Decimal,
    /// Profit factor out-of-sample
    pub out_of_sample_pf: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverfitDetectionResult {
    pub state: OverfitState,
    /// OOS / IS expectancy ratio — below 0.6 suggests severe overfit
    pub expectancy_ratio: Decimal,
    /// OOS / IS profit factor ratio
    pub pf_ratio: Decimal,
    /// How many parameters were tested per trade in the in-sample set
    pub param_density: Decimal,
    pub confidence_penalty: Decimal, // applied multiplier [0,1]
    pub reasons: Vec<String>,
}

pub struct OverfitDetector;

impl OverfitDetector {
    pub fn evaluate(input: &OverfitInput) -> OverfitDetectionResult {
        let mut reasons = Vec::new();
        let mut penalty = dec!(1);

        // OOS / IS ratios
        let expectancy_ratio = if input.in_sample_expectancy.abs() > Decimal::ZERO {
            input.out_of_sample_expectancy / input.in_sample_expectancy
        } else {
            dec!(1)
        };

        let pf_ratio = if input.in_sample_pf > Decimal::ZERO {
            input.out_of_sample_pf / input.in_sample_pf
        } else {
            dec!(1)
        };

        // Parameter density: number of variants tested per in-sample trade
        let param_density = if input.in_sample_trades > 0 {
            Decimal::from(input.parameters_tested) / Decimal::from(input.in_sample_trades)
        } else {
            dec!(1)
        };

        // Score signals
        if expectancy_ratio < dec!(0.60) {
            reasons.push(format!(
                "OOS/IS expectancy ratio critically low: {:.3}",
                expectancy_ratio
            ));
            penalty *= dec!(0.50);
        } else if expectancy_ratio < dec!(0.80) {
            reasons.push(format!(
                "OOS/IS expectancy ratio degraded: {:.3}",
                expectancy_ratio
            ));
            penalty *= dec!(0.75);
        }

        if pf_ratio < dec!(0.70) {
            reasons.push(format!(
                "OOS/IS profit factor ratio critically low: {:.3}",
                pf_ratio
            ));
            penalty *= dec!(0.60);
        } else if pf_ratio < dec!(0.85) {
            reasons.push(format!(
                "OOS/IS profit factor ratio degraded: {:.3}",
                pf_ratio
            ));
            penalty *= dec!(0.85);
        }

        if param_density > dec!(0.10) {
            reasons.push(format!(
                "Parameter density excessive: {:.3} variants/trade",
                param_density
            ));
            penalty *= dec!(0.70);
        } else if param_density > dec!(0.05) {
            reasons.push(format!(
                "Parameter density elevated: {:.3} variants/trade",
                param_density
            ));
            penalty *= dec!(0.85);
        }

        if input.out_of_sample_trades < 30 {
            reasons.push(format!(
                "Insufficient OOS trades: {}",
                input.out_of_sample_trades
            ));
            penalty *= dec!(0.60);
        }

        let state = if penalty < dec!(0.40) {
            OverfitState::Dangerous
        } else if penalty < dec!(0.60) {
            OverfitState::Overfit
        } else if penalty < dec!(0.75) {
            OverfitState::Warning
        } else if penalty < dec!(0.90) {
            OverfitState::Caution
        } else {
            OverfitState::Healthy
        };

        OverfitDetectionResult {
            state,
            expectancy_ratio,
            pf_ratio,
            param_density,
            confidence_penalty: penalty.clamp(dec!(0), dec!(1)),
            reasons,
        }
    }
}
