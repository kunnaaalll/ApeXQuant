use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum PortfolioError {
    #[error("Invariant violation: Equity ({equity}) != Balance ({balance}) + Floating PnL ({floating_pnl})")]
    InvalidEquity {
        equity: Decimal,
        balance: Decimal,
        floating_pnl: Decimal,
    },
    #[error("Invariant violation: Free Margin ({free_margin}) != Equity ({equity}) - Used Margin ({used_margin})")]
    InvalidFreeMargin {
        free_margin: Decimal,
        equity: Decimal,
        used_margin: Decimal,
    },
    #[error("Invariant violation: Margin Level cannot be negative ({margin_level})")]
    NegativeMarginLevel {
        margin_level: Decimal,
    },
    #[error("Invariant violation: Active Positions cannot be negative")]
    NegativeActivePositions,
    #[error("Invariant violation: Peak Equity ({peak_equity}) < Equity ({equity})")]
    PeakEquityLowerThanEquity {
        peak_equity: Decimal,
        equity: Decimal,
    },
    #[error("Invariant violation: Drawdown cannot be negative ({drawdown})")]
    NegativeDrawdown {
        drawdown: Decimal,
    },
    #[error("Insufficient Free Margin: Required {required}, Available {available}")]
    InsufficientMargin {
        required: Decimal,
        available: Decimal,
    },
    #[error("Invalid Transition: Cannot transition from {from:?} to {to:?}")]
    InvalidRecoveryTransition {
        from: String,
        to: String,
    },
    #[error("Position {0} not found")]
    PositionNotFound(uuid::Uuid),
}
