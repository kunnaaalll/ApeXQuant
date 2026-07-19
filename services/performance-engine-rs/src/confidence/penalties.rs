use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PenaltySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ConfidencePenalty {
    pub name: String,
    pub severity: PenaltySeverity,
    pub impact: Decimal,
    pub reason: String,
}

impl ConfidencePenalty {
    pub fn small_sample_size(trade_count: u32) -> Option<Self> {
        if trade_count < 20 {
            Some(Self {
                name: "Small Sample Size".into(),
                severity: PenaltySeverity::Critical,
                impact: dec!(50.0),
                reason: "Fewer than 20 trades provides insufficient statistical significance"
                    .into(),
            })
        } else if trade_count < 50 {
            Some(Self {
                name: "Weak Sample Size".into(),
                severity: PenaltySeverity::Medium,
                impact: dec!(20.0),
                reason: "20-50 trades provides weak statistical significance".into(),
            })
        } else {
            None
        }
    }

    pub fn large_drawdown(max_drawdown: Decimal) -> Option<Self> {
        if max_drawdown > dec!(0.2) {
            Some(Self {
                name: "Large Drawdown".into(),
                severity: PenaltySeverity::High,
                impact: dec!(30.0),
                reason: "Significant historical drawdown reduces trust".into(),
            })
        } else {
            None
        }
    }

    pub fn edge_degradation(is_degrading: bool) -> Option<Self> {
        if is_degrading {
            Some(Self {
                name: "Edge Degradation".into(),
                severity: PenaltySeverity::High,
                impact: dec!(25.0),
                reason: "Recent edge is trailing long term edge".into(),
            })
        } else {
            None
        }
    }

    pub fn high_volatility(volatility: Decimal) -> Option<Self> {
        if volatility > dec!(0.3) {
            Some(Self {
                name: "High Volatility".into(),
                severity: PenaltySeverity::Medium,
                impact: dec!(15.0),
                reason: "High volatility makes edge unpredictable".into(),
            })
        } else {
            None
        }
    }

    pub fn unstable_expectancy(is_unstable: bool) -> Option<Self> {
        if is_unstable {
            Some(Self {
                name: "Unstable Expectancy".into(),
                severity: PenaltySeverity::High,
                impact: dec!(20.0),
                reason: "Expectancy exhibits significant negative drift".into(),
            })
        } else {
            None
        }
    }

    pub fn high_variance(variance: Decimal) -> Option<Self> {
        if variance > dec!(0.5) {
            Some(Self {
                name: "High Variance".into(),
                severity: PenaltySeverity::Medium,
                impact: dec!(10.0),
                reason: "High variance in trade outcomes".into(),
            })
        } else {
            None
        }
    }

    pub fn consecutive_losses(losses: u32) -> Option<Self> {
        if losses >= 5 {
            Some(Self {
                name: "Consecutive Losses".into(),
                severity: PenaltySeverity::Medium,
                impact: dec!(15.0),
                reason: format!("Currently in a streak of {} consecutive losses", losses),
            })
        } else {
            None
        }
    }
}
