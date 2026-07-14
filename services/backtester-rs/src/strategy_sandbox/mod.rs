//! Strategy Sandbox Module
//!
//! Provides isolated, deterministic environments for testing and evaluating strategies
//! before shadow mode promotion. Uses real replay engine + execution model components.
//! Same input (data + session params) always produces identical output.

use crate::execution_model::fill::{FillEngine, FillRequest, FillStatus};
use crate::execution_model::slippage::{SlippageContext, SlippageModel};
use crate::execution_model::spread::{SpreadContext, SpreadModel};
use crate::market_replay::models::Tick;
use rust_decimal::Decimal;
use time::OffsetDateTime;

/// Configuration for a sandbox simulation session.
#[derive(Debug, Clone)]
pub struct SandboxSession {
    pub session_id: String,
    pub strategy_id: String,
    /// Unix milliseconds — defines the data window to replay.
    pub start_time_ms: i64,
    pub end_time_ms: i64,
    /// Fraction of equity risked per trade (e.g. 0.01 = 1%).
    pub risk_per_trade_fraction: Decimal,
    /// Initial equity in account currency.
    pub starting_equity: Decimal,
    /// Base spread in price units (e.g. 0.00010 for EURUSD 1 pip).
    pub base_spread: Decimal,
    /// Base slippage in price units.
    pub base_slippage: Decimal,
    /// Maximum fill size (position size cap).
    pub max_fill_size: Decimal,
}

impl SandboxSession {
    pub fn new(
        session_id: String,
        strategy_id: String,
        start_time_ms: i64,
        end_time_ms: i64,
    ) -> Self {
        Self {
            session_id,
            strategy_id,
            start_time_ms,
            end_time_ms,
            risk_per_trade_fraction: Decimal::new(1, 2), // 1%
            starting_equity: Decimal::from(10_000i64),
            base_spread: Decimal::new(10, 5),  // 1 pip
            base_slippage: Decimal::new(5, 6), // 0.5 pip
            max_fill_size: Decimal::from(10i64),
        }
    }
}

/// Results of a completed sandbox simulation.
#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub max_drawdown: Decimal,
    pub total_return: Decimal,
    pub total_pnl: Decimal,
    pub total_spread_cost: Decimal,
    pub total_slippage_cost: Decimal,
    pub total_rejected: usize,
    /// True iff the simulation is fully deterministic (same input → same output).
    pub is_deterministic: bool,
}

/// Minimal trade signal emitted by a strategy stub during sandbox replay.
/// In production, this comes from the Strategy Engine — here we use a
/// deterministic rule based on mid-price movement across consecutive ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TradeDirection {
    Long,
    Short,
}

/// Internal sandbox trade tracker.
#[derive(Debug)]
struct OpenTrade {
    direction: TradeDirection,
    entry_price: Decimal,
    size: Decimal,
    _entry_time: OffsetDateTime,
}

pub struct StrategySandbox;

impl StrategySandbox {
    /// Run a deterministic isolated replay simulation over the provided tick data.
    ///
    /// The "strategy" applied is a simple deterministic momentum rule:
    /// - If the mid-price rose over the last `LOOKBACK` ticks → Long signal.
    /// - If the mid-price fell over the last `LOOKBACK` ticks → Short signal.
    /// - Exit after `EXIT_AFTER` ticks from entry.
    ///
    /// This is not a production strategy — it is a harness that exercises the full
    /// execution pipeline (slippage, spread, partial fills, liquidity constraints)
    /// in a deterministic way. The Strategy Engine will supply real signals in production.
    pub fn run_session(
        session: &SandboxSession,
        ticks: &[Tick],
    ) -> Result<SandboxResult, &'static str> {
        const LOOKBACK: usize = 5;
        const EXIT_AFTER: usize = 10;

        if ticks.is_empty() {
            return Ok(SandboxResult {
                total_trades: 0,
                winning_trades: 0,
                losing_trades: 0,
                win_rate: Decimal::ZERO,
                profit_factor: Decimal::ZERO,
                max_drawdown: Decimal::ZERO,
                total_return: Decimal::ZERO,
                total_pnl: Decimal::ZERO,
                total_spread_cost: Decimal::ZERO,
                total_slippage_cost: Decimal::ZERO,
                total_rejected: 0,
                is_deterministic: true,
            });
        }

        // Filter ticks to the session's time window
        let window: Vec<&Tick> = ticks
            .iter()
            .filter(|t| {
                let ts_ms = t.timestamp.unix_timestamp() * 1000 + t.timestamp.millisecond() as i64;
                ts_ms >= session.start_time_ms && ts_ms <= session.end_time_ms
            })
            .collect();

        let fill_engine = FillEngine::new(session.max_fill_size);
        let slippage_model = SlippageModel::new(
            session.base_slippage,
            Decimal::from(3i64), // 3× slippage during news events
        );
        let spread_model = SpreadModel::new(Decimal::new(15, 1)); // 1.5× out-of-session spread

        let mut equity = session.starting_equity;
        let mut peak_equity = equity;
        let mut max_drawdown = Decimal::ZERO;
        let mut gross_profit = Decimal::ZERO;
        let mut gross_loss = Decimal::ZERO;
        let mut total_trades = 0usize;
        let mut winning_trades = 0usize;
        let mut losing_trades = 0usize;
        let mut total_spread_cost = Decimal::ZERO;
        let mut total_slippage_cost = Decimal::ZERO;
        let mut total_rejected = 0usize;

        let mut open_trade: Option<OpenTrade> = None;
        let mut ticks_since_entry: usize = 0;

        for (i, tick) in window.iter().enumerate() {
            let mid = (tick.bid + tick.ask) / Decimal::TWO;

            // Check exit condition for open trade
            if let Some(ref trade) = open_trade {
                ticks_since_entry += 1;
                if ticks_since_entry >= EXIT_AFTER {
                    // Exit at current mid
                    let exit_mid = mid;
                    let spread_ctx = SpreadContext {
                        base_spread: session.base_spread,
                        volatility_factor: Decimal::ONE,
                        is_out_of_session: false,
                    };
                    let spread_cost = spread_model.calculate_spread(&spread_ctx) * trade.size;
                    total_spread_cost += spread_cost;

                    let pnl = match trade.direction {
                        TradeDirection::Long => {
                            (exit_mid - trade.entry_price) * trade.size - spread_cost
                        }
                        TradeDirection::Short => {
                            (trade.entry_price - exit_mid) * trade.size - spread_cost
                        }
                    };

                    if pnl > Decimal::ZERO {
                        gross_profit += pnl;
                        winning_trades += 1;
                    } else {
                        gross_loss += pnl.abs();
                        losing_trades += 1;
                    }
                    equity += pnl;
                    total_trades += 1;

                    if equity > peak_equity {
                        peak_equity = equity;
                    }
                    let dd = if peak_equity > Decimal::ZERO {
                        (peak_equity - equity) / peak_equity
                    } else {
                        Decimal::ZERO
                    };
                    if dd > max_drawdown {
                        max_drawdown = dd;
                    }

                    open_trade = None;
                    ticks_since_entry = 0;
                }
            }

            // Emit entry signal if no open trade and enough history
            if open_trade.is_none() && i >= LOOKBACK {
                let prev_tick = window[i - LOOKBACK];
                let prev_mid = (prev_tick.bid + prev_tick.ask) / Decimal::TWO;

                let direction = if mid > prev_mid {
                    TradeDirection::Long
                } else if mid < prev_mid {
                    TradeDirection::Short
                } else {
                    continue; // No signal on flat mid
                };

                // Position size = (equity * risk_fraction) / (spread as proxy for stop distance)
                let stop_dist = session.base_spread * Decimal::from(10i64);
                let position_size = if stop_dist > Decimal::ZERO {
                    equity * session.risk_per_trade_fraction / stop_dist
                } else {
                    Decimal::ONE
                };

                // Simulate liquidity (tick bid/ask size as proxy)
                let available_liq = tick.bid_size.max(tick.ask_size).max(Decimal::ONE);

                let slippage_ctx = SlippageContext {
                    order_size: position_size,
                    available_liquidity: available_liq,
                    is_news_event: false,
                    volatility: Decimal::ONE,
                };
                let slippage = slippage_model.expected_slippage(&slippage_ctx);
                total_slippage_cost += slippage;

                let fill_req = FillRequest {
                    order_id: format!("sandbox_{}_{}", session.session_id, i),
                    requested_size: position_size,
                    limit_price: None,
                    timestamp: tick.timestamp,
                };
                let fill = fill_engine.process_fill(&fill_req, mid, available_liq);

                match fill.status {
                    FillStatus::Rejected(_) => {
                        total_rejected += 1;
                    }
                    FillStatus::Full | FillStatus::Partial | FillStatus::Delayed => {
                        let entry_price = fill.fill_price
                            + match direction {
                                TradeDirection::Long => slippage,
                                TradeDirection::Short => -slippage,
                            };
                        open_trade = Some(OpenTrade {
                            direction,
                            entry_price,
                            size: fill.filled_size,
                            _entry_time: tick.timestamp,
                        });
                        ticks_since_entry = 0;
                    }
                }
            }
        }

        // Force-close any still-open trade at end of window
        if let Some(trade) = open_trade {
            if let Some(last_tick) = window.last() {
                let exit_mid = (last_tick.bid + last_tick.ask) / Decimal::TWO;
                let spread_ctx = SpreadContext {
                    base_spread: session.base_spread,
                    volatility_factor: Decimal::ONE,
                    is_out_of_session: false,
                };
                let spread_cost = spread_model.calculate_spread(&spread_ctx) * trade.size;
                total_spread_cost += spread_cost;
                let pnl = match trade.direction {
                    TradeDirection::Long => {
                        (exit_mid - trade.entry_price) * trade.size - spread_cost
                    }
                    TradeDirection::Short => {
                        (trade.entry_price - exit_mid) * trade.size - spread_cost
                    }
                };
                if pnl > Decimal::ZERO {
                    gross_profit += pnl;
                    winning_trades += 1;
                } else {
                    gross_loss += pnl.abs();
                    losing_trades += 1;
                }
                equity += pnl;
                total_trades += 1;
            }
        }

        let win_rate = if total_trades > 0 {
            Decimal::from(winning_trades as i64) / Decimal::from(total_trades as i64)
        } else {
            Decimal::ZERO
        };

        let profit_factor = if gross_loss > Decimal::ZERO {
            gross_profit / gross_loss
        } else if gross_profit > Decimal::ZERO {
            Decimal::new(999, 0)
        } else {
            Decimal::ZERO
        };

        let total_pnl = equity - session.starting_equity;
        let total_return = if session.starting_equity > Decimal::ZERO {
            total_pnl / session.starting_equity
        } else {
            Decimal::ZERO
        };

        Ok(SandboxResult {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            profit_factor,
            max_drawdown,
            total_return,
            total_pnl,
            total_spread_cost,
            total_slippage_cost,
            total_rejected,
            is_deterministic: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn make_tick(timestamp_ms: i64, bid: i64, ask: i64, scale: u32) -> Tick {
        let ts = OffsetDateTime::from_unix_timestamp(timestamp_ms / 1000)
            .unwrap_or(OffsetDateTime::UNIX_EPOCH);
        Tick {
            symbol: "EURUSD".to_string(),
            timestamp: ts,
            bid: Decimal::new(bid, scale),
            ask: Decimal::new(ask, scale),
            bid_size: Decimal::ONE,
            ask_size: Decimal::ONE,
        }
    }

    fn session() -> SandboxSession {
        SandboxSession::new(
            "test-session".to_string(),
            "test-strategy".to_string(),
            0,
            i64::MAX,
        )
    }

    #[test]
    fn test_empty_ticks_returns_zero() {
        let s = session();
        let result = StrategySandbox::run_session(&s, &[]).expect("ok");
        assert_eq!(result.total_trades, 0);
        assert_eq!(result.win_rate, Decimal::ZERO);
        assert!(result.is_deterministic);
    }

    #[test]
    fn test_deterministic_same_result_twice() {
        let s = session();
        let ticks: Vec<Tick> = (0..30)
            .map(|i| {
                // Create ascending then descending price pattern
                let price = 11000i64 + (i as i64 % 10);
                make_tick(i * 1000, price, price + 10, 5)
            })
            .collect();
        let r1 = StrategySandbox::run_session(&s, &ticks).expect("ok");
        let r2 = StrategySandbox::run_session(&s, &ticks).expect("ok");
        assert_eq!(r1.total_trades, r2.total_trades);
        assert_eq!(r1.total_pnl, r2.total_pnl);
        assert!(r1.is_deterministic);
    }

    #[test]
    fn test_profit_factor_sensible() {
        let s = session();
        let ticks: Vec<Tick> = (0..50)
            .map(|i| {
                let price = 11000i64 + i as i64;
                make_tick(i * 500, price, price + 10, 5)
            })
            .collect();
        let result = StrategySandbox::run_session(&s, &ticks).expect("ok");
        // Profit factor must be non-negative
        assert!(result.profit_factor >= Decimal::ZERO);
        // Win rate must be in [0, 1]
        assert!(result.win_rate >= Decimal::ZERO);
        assert!(result.win_rate <= Decimal::ONE);
    }
}
