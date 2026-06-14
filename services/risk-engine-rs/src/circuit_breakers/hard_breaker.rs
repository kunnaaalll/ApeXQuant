//! Hard circuit breakers - completely stop trading
use rust_decimal::prelude::FromPrimitive;

use super::{
    BreakerSeverity, CircuitBreaker, CircuitBreakerState, CircuitBreakerTrigger,
};
use crate::RiskInputs;
use rust_decimal::Decimal;
use std::time::{Duration, Instant};
use time::OffsetDateTime;

/// Hard circuit breaker - stops all new trading
pub struct HardBreaker {
    name: String,
    check_fn: Box<dyn Fn(&RiskInputs) -> Option<String> + Send + Sync>,
    state: CircuitBreakerState,
    triggered_at: Option<Instant>,
    severity: BreakerSeverity,
}

impl HardBreaker {
    /// Create custom hard breaker
    pub fn new<F>(name: &str, check_fn: F, severity: BreakerSeverity) -> Self
    where
        F: Fn(&RiskInputs) -> Option<String> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            check_fn: Box::new(check_fn),
            state: CircuitBreakerState::Normal,
            triggered_at: None,
            severity,
        }
    }

    /// Create drawdown-based hard breaker
    pub fn drawdown(hard_limit: Decimal, soft_limit: Decimal) -> Self {
        HardBreaker::new(
            "drawdown_limit",
            move |inputs| {
                // Calculate drawdown as percent of peak
                let peak = inputs.equity.max(inputs.balance);
                if peak > Decimal::ZERO {
                    let current_dd = (peak - inputs.equity) / peak;

                    if current_dd >= hard_limit {
                        Some(format!(
                            "Hard drawdown limit reached: {:.1}% (limit: {:.1}%)",
                            current_dd * Decimal::from(100),
                            hard_limit * Decimal::from(100)
                        ))
                    } else if current_dd >= soft_limit {
                        Some(format!(
                            "Drawdown warning: {:.1}% (soft limit: {:.1}%)",
                            current_dd * Decimal::from(100),
                            soft_limit * Decimal::from(100)
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            BreakerSeverity::Critical,
        )
    }

    /// Create daily loss limit breaker
    pub fn daily_loss(daily_limit: Decimal) -> Self {
        HardBreaker::new(
            "daily_loss_limit",
            move |inputs| {
                let peak = inputs.equity.max(inputs.balance);
                if peak > Decimal::ZERO {
                    let daily_dd = inputs.daily_pnl.abs() / peak;

                    if daily_dd >= daily_limit {
                        Some(format!(
                            "Daily loss limit reached: {:.1}% (limit: {:.1}%)",
                            daily_dd * Decimal::from(100),
                            daily_limit * Decimal::from(100)
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            BreakerSeverity::Critical,
        )
    }

    /// Create catastrophic loss breaker
    pub fn catastrophic_loss(catastrophic_pct: Decimal) -> Self {
        HardBreaker::new(
            "catastrophic_loss",
            move |inputs| {
                let peak = inputs.equity.max(inputs.balance);
                if peak > Decimal::ZERO {
                    let loss = (peak - inputs.equity) / peak;

                    if loss >= catastrophic_pct {
                        Some(format!(
                            "CATASTROPHIC LOSS DETECTED: {:.1}% - ALL TRADING HALTED",
                            loss * Decimal::from(100)
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            BreakerSeverity::Catastrophic,
        )
    }

    /// Create execution failure breaker
    pub fn execution_failures(max_failures: u32) -> Self {
        HardBreaker::new(
            "execution_failures",
            move |_inputs| {
                // In real implementation, would check execution stats
                // For now, simplified
                None
            },
            BreakerSeverity::Critical,
        )
    }

    /// Create data quality breaker
    pub fn data_quality() -> Self {
        HardBreaker::new(
            "data_quality",
            move |inputs| {
                // Check for clearly invalid data
                if inputs.equity <= Decimal::ZERO {
                    Some("Invalid equity data: zero or negative".to_string())
                } else if inputs.spread > Decimal::from_f64(0.01).unwrap() {
                    // Spread > 100 pips suggests data error
                    Some(format!(
                        "Suspect spread data: {} (> 100 pips)",
                        inputs.spread
                    ))
                } else if inputs.entry_price <= Decimal::ZERO {
                    Some("Invalid price data: zero or negative".to_string())
                } else {
                    None
                }
            },
            BreakerSeverity::Critical,
        )
    }

    /// Create margin call proximity breaker
    pub fn margin_proximity(margin_pct: Decimal) -> Self {
        HardBreaker::new(
            "margin_proximity",
            move |inputs| {
                // Simplified - would check actual margin used
                let exposure = inputs.open_positions.iter()
                    .map(|(_, size, _)| size.abs())
                    .sum::<Decimal>();

                let margin_used = exposure * Decimal::from_f64(0.02).unwrap(); // Assume 50:1 leverage
                let margin_pct_calc = if inputs.equity > Decimal::ZERO {
                    margin_used / inputs.equity
                } else {
                    Decimal::ZERO
                };

                if margin_pct_calc >= margin_pct {
                    Some(format!(
                        "Margin usage near limit: {:.1}% (threshold: {:.1}%)",
                        margin_pct_calc * Decimal::from(100),
                        margin_pct * Decimal::from(100)
                    ))
                } else {
                    None
                }
            },
            BreakerSeverity::Critical,
        )
    }
}

impl CircuitBreaker for HardBreaker {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self, inputs: &RiskInputs) -> Option<CircuitBreakerTrigger> {
        (self.check_fn)(inputs).map(|reason| CircuitBreakerTrigger {
            breaker: self.name.clone(),
            reason,
            severity: self.severity.clone(),
            recommended_state: CircuitBreakerState::HardStop,
            timestamp: OffsetDateTime::now_utc(),
            duration: Duration::from_secs(3600), // 1 hour default for hard breakers
        })
    }

    fn state(&self) -> CircuitBreakerState {
        self.state
    }

    fn reset(&mut self) {
        self.state = CircuitBreakerState::Normal;
        self.triggered_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarketSession;
    use time::OffsetDateTime;

    fn test_inputs() -> RiskInputs {
        RiskInputs {
            equity: Decimal::from(10000),
            balance: Decimal::from(10000),
            symbol: "EURUSD".to_string(),
            direction: 1,
            entry_price: Decimal::from_str("1.08500").unwrap().try_into().unwrap(),
            stop_loss: Decimal::from_str("1.08200").unwrap().try_into().unwrap(),
            take_profit: None,
            signal_confidence: Decimal::from_f64(0.8).unwrap(),
            confluence_score: Decimal::from(7),
            regime_quality: Decimal::from_f64(0.7).unwrap(),
            pattern_quality: Decimal::from_f64(0.75).unwrap(),
            atr: None,
            spread: Decimal::from_f64(0.0001).unwrap(),
            open_positions: vec![],
            daily_pnl: Decimal::ZERO,
            daily_trades: 0,
            recent_trades: vec![],
            session: MarketSession::London,
        }
    }

    fn inputs_with_loss(equity: Decimal, peak: Decimal) -> RiskInputs {
        RiskInputs {
            equity,
            balance: peak,
            symbol: "EURUSD".to_string(),
            direction: 1,
            entry_price: Decimal::from_str("1.08500").unwrap().try_into().unwrap(),
            stop_loss: Decimal::from_str("1.08200").unwrap().try_into().unwrap(),
            take_profit: None,
            signal_confidence: Decimal::from_f64(0.8).unwrap(),
            confluence_score: Decimal::from(7),
            regime_quality: Decimal::from_f64(0.7).unwrap(),
            pattern_quality: Decimal::from_f64(0.75).unwrap(),
            atr: None,
            spread: Decimal::from_f64(0.0001).unwrap(),
            open_positions: vec![],
            daily_pnl: Decimal::ZERO,
            daily_trades: 0,
            recent_trades: vec![],
            session: MarketSession::London,
        }
    }

    #[test]
    fn test_drawdown_breaker_triggered() {
        let breaker = HardBreaker::drawdown(
            Decimal::from_f64(0.10).unwrap(),
            Decimal::from_f64(0.05).unwrap(),
        );

        // 15% drawdown
        let inputs = inputs_with_loss(
            Decimal::from(8500),
            Decimal::from(10000),
        );

        let trigger = breaker.check(&inputs);
        assert!(trigger.is_some());

        let t = trigger.unwrap();
        assert_eq!(t.severity, BreakerSeverity::Critical);
        assert_eq!(t.recommended_state, CircuitBreakerState::HardStop);
    }

    #[test]
    fn test_catastrophic_loss() {
        let breaker = HardBreaker::catastrophic_loss(Decimal::from_f64(0.50).unwrap());

        // 60% loss
        let inputs = inputs_with_loss(
            Decimal::from(4000),
            Decimal::from(10000),
        );

        let trigger = breaker.check(&inputs);
        assert!(trigger.is_some());
        assert!(trigger.unwrap().reason.contains("CATASTROPHIC"));
    }

    #[test]
    fn test_data_quality() {
        let breaker = HardBreaker::data_quality();

        let mut inputs = test_inputs();
        inputs.equity = Decimal::ZERO;

        let trigger = breaker.check(&inputs);
        assert!(trigger.is_some());
        assert!(trigger.unwrap().reason.contains("Invalid equity"));
    }

    #[test]
    fn test_daily_loss_limit() {
        let breaker = HardBreaker::daily_loss(Decimal::from_f64(0.05).unwrap());

        let mut inputs = test_inputs();
        inputs.daily_pnl = Decimal::from(-600); // > 5% of 10k

        let trigger = breaker.check(&inputs);
        assert!(trigger.is_some());
    }

    #[test]
    fn test_no_trigger_on_normal() {
        let breaker = HardBreaker::drawdown(
            Decimal::from_f64(0.20).unwrap(),
            Decimal::from_f64(0.10).unwrap(),
        );

        let inputs = test_inputs();
        assert!(breaker.check(&inputs).is_none());
    }
}
