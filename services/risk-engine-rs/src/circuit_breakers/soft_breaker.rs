//! Soft circuit breakers - reduce risk but don't stop completely
use rust_decimal::prelude::FromPrimitive;

use crate::{
    circuit_breakers::{BreakerSeverity, CircuitBreaker, CircuitBreakerState, CircuitBreakerTrigger},
    RiskInputs, TradeResult,
};
use rust_decimal::Decimal;
use std::time::{Duration, Instant};
use time::OffsetDateTime;

/// Soft circuit breaker implementation
pub struct SoftBreaker {
    name: String,
    check_fn: Box<dyn Fn(&RiskInputs) -> Option<String> + Send + Sync>,
    state: CircuitBreakerState,
    triggered_at: Option<Instant>,
    cooldown: Duration,
}

impl SoftBreaker {
    /// Create custom soft breaker
    pub fn new<F>(name: &str, check_fn: F, cooldown_secs: u64) -> Self
    where
        F: Fn(&RiskInputs) -> Option<String> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            check_fn: Box::new(check_fn),
            state: CircuitBreakerState::Normal,
            triggered_at: None,
            cooldown: Duration::from_secs(cooldown_secs),
        }
    }

    /// Create consecutive loss breaker
    pub fn consecutive_losses(threshold: u32) -> Self {
        SoftBreaker::new(
            "consecutive_losses",
            move |inputs| {
                let recent_losses = count_consecutive_losses(&inputs.recent_trades);
                if recent_losses >= threshold {
                    Some(format!(
                        "{} consecutive losses detected (threshold: {})",
                        recent_losses, threshold
                    ))
                } else {
                    None
                }
            },
            1800, // 30 min cooldown
        )
    }

    /// Create exposure concentration breaker
    pub fn exposure_concentration(correlation_threshold: Decimal) -> Self {
        SoftBreaker::new(
            "exposure_concentration",
            move |_inputs| {
                // Simplified - real implementation would check actual exposure
                None
            },
            900, // 15 min cooldown
        )
    }

    /// Create spread anomaly breaker
    pub fn spread_anomaly(max_spread_multiple: Decimal) -> Self {
        SoftBreaker::new(
            "spread_anomaly",
            move |inputs| {
                let normal_spread = Decimal::from_f64(0.0001).unwrap();
                if normal_spread > Decimal::ZERO {
                    let multiple = inputs.spread / normal_spread;
                    if multiple > max_spread_multiple {
                        Some(format!(
                            "Spread expanded {}x normal (threshold: {}x)",
                            multiple, max_spread_multiple
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            600, // 10 min cooldown
        )
    }

    /// Create volatility spike breaker
    pub fn volatility_spike(atr_multiple: Decimal) -> Self {
        SoftBreaker::new(
            "volatility_spike",
            move |inputs| {
                if let Some(atr) = inputs.atr {
                    let normal_atr = Decimal::from_f64(0.0005).unwrap();
                    if normal_atr > Decimal::ZERO {
                        let multiple = atr / normal_atr;
                        if multiple > atr_multiple {
                            Some(format!(
                                "ATR expanded {}x normal (threshold: {}x)",
                                multiple, atr_multiple
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            1200, // 20 min cooldown
        )
    }

    /// Create win rate degradation breaker
    pub fn win_rate_degradation(min_trades: usize, min_win_rate: Decimal) -> Self {
        SoftBreaker::new(
            "win_rate_degradation",
            move |inputs| {
                if inputs.recent_trades.len() >= min_trades {
                    let wins = inputs.recent_trades.iter().filter(|t| t.is_win).count();
                    let win_rate = Decimal::from(wins as u32) / Decimal::from(inputs.recent_trades.len() as u32);

                    if win_rate < min_win_rate {
                        Some(format!(
                            "Win rate degraded to {:.1}% (minimum: {:.1}%)",
                            win_rate * Decimal::from(100),
                            min_win_rate * Decimal::from(100)
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            3600, // 1 hour cooldown
        )
    }

    fn check_cooldown(&self) -> bool {
        match self.triggered_at {
            Some(at) => at.elapsed() > self.cooldown,
            None => true,
        }
    }
}

impl CircuitBreaker for SoftBreaker {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self, inputs: &RiskInputs) -> Option<CircuitBreakerTrigger> {
        if !self.check_cooldown() {
            return None;
        }

        (self.check_fn)(inputs).map(|reason| CircuitBreakerTrigger {
            breaker: self.name.clone(),
            reason,
            severity: BreakerSeverity::Warning,
            recommended_state: CircuitBreakerState::ReducedRisk,
            timestamp: OffsetDateTime::now_utc(),
            duration: self.cooldown,
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

fn count_consecutive_losses(trades: &[TradeResult]) -> u32 {
    trades.iter().rev().take_while(|t| !t.is_win).count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarketSession;
    use time::OffsetDateTime;

    fn test_inputs_with_trades(trades: Vec<TradeResult>) -> RiskInputs {
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
            recent_trades: trades,
            session: MarketSession::London,
        }
    }

    #[test]
    fn test_consecutive_losses_breaker() {
        let breaker = SoftBreaker::consecutive_losses(3);
        let trades: Vec<TradeResult> = (0..5)
            .map(|_| TradeResult {
                pnl: Decimal::from(-50),
                is_win: false,
                duration_min: 30,
                timestamp: OffsetDateTime::now_utc(),
            })
            .collect();

        let inputs = test_inputs_with_trades(trades);
        let trigger = breaker.check(&inputs);

        assert!(trigger.is_some());
        let t = trigger.unwrap();
        assert_eq!(t.severity, BreakerSeverity::Warning);
        assert!(t.reason.contains("5 consecutive losses"));
    }

    #[test]
    fn test_spread_anomaly() {
        let breaker = SoftBreaker::spread_anomaly(Decimal::from(3));

        let mut inputs = test_inputs_with_trades(vec![]);
        inputs.spread = Decimal::from_f64(0.0005).unwrap(); // 5x normal

        let trigger = breaker.check(&inputs);
        assert!(trigger.is_some());
    }

    #[test]
    fn test_no_trigger_on_normal() {
        let breaker = SoftBreaker::consecutive_losses(5);
        let inputs = test_inputs_with_trades(vec![]);

        assert!(breaker.check(&inputs).is_none());
    }
}
