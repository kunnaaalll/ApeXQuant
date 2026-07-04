use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct DailyLimits {
    pub max_trades_per_day: u32,
    pub max_daily_drawdown: Decimal,
    pub max_daily_volume: Decimal,
}

pub struct DailyUsage {
    pub trades_count: u32,
    pub current_drawdown: Decimal,
    pub volume_traded: Decimal,
}

impl DailyLimits {
    pub fn new(max_trades: u32, max_drawdown: Decimal, max_volume: Decimal) -> Self {
        Self {
            max_trades_per_day: max_trades,
            max_daily_drawdown: max_drawdown,
            max_daily_volume: max_volume,
        }
    }

    /// Check if submitting this order exceeds limits
    pub fn is_limit_exceeded(&self, usage: &DailyUsage, order_volume: Decimal) -> bool {
        if usage.trades_count >= self.max_trades_per_day {
            return true;
        }

        if usage.current_drawdown >= self.max_daily_drawdown {
            return true;
        }

        if usage.volume_traded + order_volume > self.max_daily_volume {
            return true;
        }

        false
    }
}
