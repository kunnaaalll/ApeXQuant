use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// A consecutive-loss or consecutive-ruin series input for collapse detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseInput {
    /// Expectancy over most recent N trades
    pub recent_expectancy: Decimal,
    /// Win rate over most recent N trades
    pub recent_win_rate: Decimal,
    /// Profit factor over most recent N trades
    pub recent_profit_factor: Decimal,
    /// Number of consecutive losses
    pub consecutive_losses: u32,
    /// Current drawdown as positive fraction (0.0 = no drawdown, 1.0 = total loss)
    pub current_drawdown_pct: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollapseSignal {
    /// No distress detected
    Clear,
    /// Early warning – monitor closely
    Caution,
    /// Immediate risk of edge failure
    Imminent,
    /// Edge has collapsed – halt strategy
    Triggered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseDetectionResult {
    pub signal: CollapseSignal,
    pub reasons: Vec<String>,
    pub severity_score: Decimal, // 0–100
}

pub struct CollapseDetector;

impl CollapseDetector {
    /// Collapse must trigger immediately on severe breach; never delayed.
    pub fn evaluate(input: &CollapseInput) -> CollapseDetectionResult {
        let mut reasons = Vec::new();
        let mut severity = dec!(0);

        if input.recent_expectancy < dec!(-0.10) {
            reasons.push(format!("Expectancy collapsed to {}", input.recent_expectancy));
            severity += dec!(35);
        } else if input.recent_expectancy < dec!(0) {
            reasons.push(format!("Expectancy negative: {}", input.recent_expectancy));
            severity += dec!(20);
        }

        if input.recent_profit_factor < dec!(0.80) {
            reasons.push(format!("Profit factor critical: {}", input.recent_profit_factor));
            severity += dec!(30);
        } else if input.recent_profit_factor < dec!(1.0) {
            reasons.push(format!("Profit factor sub-1: {}", input.recent_profit_factor));
            severity += dec!(15);
        }

        if input.recent_win_rate < dec!(0.25) {
            reasons.push(format!("Win rate critically low: {}", input.recent_win_rate));
            severity += dec!(20);
        }

        if input.consecutive_losses >= 10 {
            reasons.push(format!("{} consecutive losses", input.consecutive_losses));
            severity += dec!(20);
        } else if input.consecutive_losses >= 5 {
            reasons.push(format!("{} consecutive losses", input.consecutive_losses));
            severity += dec!(10);
        }

        if input.current_drawdown_pct >= dec!(0.20) {
            reasons.push(format!("Drawdown at {}%", input.current_drawdown_pct * dec!(100)));
            severity += dec!(20);
        }

        let signal = if severity >= dec!(70) {
            CollapseSignal::Triggered
        } else if severity >= dec!(50) {
            CollapseSignal::Imminent
        } else if severity >= dec!(25) {
            CollapseSignal::Caution
        } else {
            CollapseSignal::Clear
        };

        // Clamp to 100
        let severity_score = severity.min(dec!(100));

        CollapseDetectionResult {
            signal,
            reasons,
            severity_score,
        }
    }
}
