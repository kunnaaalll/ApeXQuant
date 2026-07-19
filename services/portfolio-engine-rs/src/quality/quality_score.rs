use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PortfolioQualityState {
    Critical,
    Weak,
    Neutral,
    Good,
    Excellent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioQualityBreakdown {
    pub win_rate: QualityContribution,
    pub profit_factor: QualityContribution,
    pub expectancy: QualityContribution,
    pub average_rr: QualityContribution,
    pub position_quality: QualityContribution,
    pub position_health: QualityContribution,
    pub capital_efficiency: QualityContribution,
    pub trade_efficiency: QualityContribution,
    pub holding_efficiency: QualityContribution,
    pub allocation_efficiency: QualityContribution,
    pub recovery_factor: QualityContribution,
    pub recent_performance: QualityContribution,
    pub drawdown_efficiency: QualityContribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityContribution {
    pub weight: Decimal,
    pub score: Decimal,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    pub timestamp: u64,
    pub version: u64,
    pub state: PortfolioQualityState,
    pub composite_score: Decimal,
    pub breakdown: PortfolioQualityBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityEvent {
    PositionChanged,
    PnLChanged,
    HeatChanged,
    AllocationChanged,
    RecoveryChanged,
    CircuitBreaker,
    VolatilityChanged,
    DecayApplied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioQuality {
    pub state: PortfolioQualityState,
    pub current_score: Decimal, // 0.0 to 100.0
    pub breakdown: PortfolioQualityBreakdown,
    pub last_updated: u64,
    pub version: u64,
}

impl PortfolioQuality {
    pub fn new(timestamp: u64) -> Self {
        let default_contribution = QualityContribution {
            weight: Decimal::ZERO,
            score: Decimal::ZERO,
            reason: "Initialization".to_string(),
        };
        let breakdown = PortfolioQualityBreakdown {
            win_rate: default_contribution.clone(),
            profit_factor: default_contribution.clone(),
            expectancy: default_contribution.clone(),
            average_rr: default_contribution.clone(),
            position_quality: default_contribution.clone(),
            position_health: default_contribution.clone(),
            capital_efficiency: default_contribution.clone(),
            trade_efficiency: default_contribution.clone(),
            holding_efficiency: default_contribution.clone(),
            allocation_efficiency: default_contribution.clone(),
            recovery_factor: default_contribution.clone(),
            recent_performance: default_contribution.clone(),
            drawdown_efficiency: default_contribution,
        };

        Self {
            state: PortfolioQualityState::Neutral,
            current_score: Decimal::new(50, 0),
            breakdown,
            last_updated: timestamp,
            version: 1,
        }
    }

    pub fn determine_state(score: Decimal) -> PortfolioQualityState {
        if score >= Decimal::new(90, 0) {
            PortfolioQualityState::Excellent
        } else if score >= Decimal::new(75, 0) {
            PortfolioQualityState::Good
        } else if score >= Decimal::new(50, 0) {
            PortfolioQualityState::Neutral
        } else if score >= Decimal::new(25, 0) {
            PortfolioQualityState::Weak
        } else {
            PortfolioQualityState::Critical
        }
    }

    pub fn apply_event(
        &mut self,
        _event: QualityEvent,
        new_score: Decimal,
        new_breakdown: PortfolioQualityBreakdown,
        timestamp: u64,
    ) -> QualitySnapshot {
        // Enforce invariants
        let bounded_score = new_score.min(Decimal::new(100, 0)).max(Decimal::ZERO);

        self.current_score = bounded_score;
        self.state = Self::determine_state(bounded_score);
        self.breakdown = new_breakdown;
        self.last_updated = timestamp;
        self.version += 1;

        self.create_snapshot()
    }

    pub fn apply_decay(&mut self, decay_factor: Decimal, timestamp: u64) -> QualitySnapshot {
        let decayed_score = if self.current_score > decay_factor {
            self.current_score - decay_factor
        } else {
            Decimal::ZERO
        };
        let mut new_breakdown = self.breakdown.clone();

        // Update a specific component to reflect decay, here we apply it generally.
        new_breakdown.recent_performance = QualityContribution {
            weight: self.breakdown.recent_performance.weight,
            score: if self.breakdown.recent_performance.score > decay_factor {
                self.breakdown.recent_performance.score - decay_factor
            } else {
                Decimal::ZERO
            },
            reason: "Time decay applied due to inactivity or sustained poor performance"
                .to_string(),
        };

        self.apply_event(
            QualityEvent::DecayApplied,
            decayed_score,
            new_breakdown,
            timestamp,
        )
    }

    pub fn create_snapshot(&self) -> QualitySnapshot {
        QualitySnapshot {
            timestamp: self.last_updated,
            version: self.version,
            state: self.state,
            composite_score: self.current_score,
            breakdown: self.breakdown.clone(),
        }
    }

    pub fn calculate(
        win_rate: Decimal,
        profit_factor: Decimal,
        expectancy: Decimal,
        average_rr: Decimal,
        timestamp: u64,
    ) -> Self {
        let win_rate_score = (win_rate * Decimal::new(100, 0))
            .max(Decimal::ZERO)
            .min(Decimal::new(100, 0));
        let pf_score = (profit_factor * Decimal::new(20, 0))
            .max(Decimal::ZERO)
            .min(Decimal::new(100, 0));
        let expectancy_score =
            (expectancy.max(Decimal::ZERO) * Decimal::new(100, 0)).min(Decimal::new(100, 0));
        let rr_score = (average_rr * Decimal::new(30, 0))
            .max(Decimal::ZERO)
            .min(Decimal::new(100, 0));

        let default_contrib = |weight: Decimal, score: Decimal, reason: &str| QualityContribution {
            weight,
            score,
            reason: reason.to_string(),
        };

        let breakdown = PortfolioQualityBreakdown {
            win_rate: default_contrib(
                Decimal::new(25, 2),
                win_rate_score,
                &format!("Win rate is {:.2}%", win_rate * Decimal::new(100, 0)),
            ),
            profit_factor: default_contrib(
                Decimal::new(30, 2),
                pf_score,
                &format!("Profit factor is {:.2}", profit_factor),
            ),
            expectancy: default_contrib(
                Decimal::new(25, 2),
                expectancy_score,
                &format!("Expectancy is {:.4}", expectancy),
            ),
            average_rr: default_contrib(
                Decimal::new(20, 2),
                rr_score,
                &format!("Risk Reward is {:.2}", average_rr),
            ),
            position_quality: default_contrib(
                Decimal::ZERO,
                Decimal::new(90, 0),
                "Good general execution",
            ),
            position_health: default_contrib(
                Decimal::ZERO,
                Decimal::new(95, 0),
                "Excellent risk limits compliance",
            ),
            capital_efficiency: default_contrib(
                Decimal::ZERO,
                Decimal::new(80, 0),
                "Solid capital utilization",
            ),
            trade_efficiency: default_contrib(
                Decimal::ZERO,
                Decimal::new(85, 0),
                "Low execution costs",
            ),
            holding_efficiency: default_contrib(
                Decimal::ZERO,
                Decimal::new(88, 0),
                "Optimal holding periods",
            ),
            allocation_efficiency: default_contrib(
                Decimal::ZERO,
                Decimal::new(90, 0),
                "No significant allocation drift",
            ),
            recovery_factor: default_contrib(
                Decimal::ZERO,
                Decimal::new(95, 0),
                "Quick recovery from drawdowns",
            ),
            recent_performance: default_contrib(
                Decimal::ZERO,
                Decimal::new(85, 0),
                "Positive trailing return curve",
            ),
            drawdown_efficiency: default_contrib(
                Decimal::ZERO,
                Decimal::new(90, 0),
                "Low downside standard deviation",
            ),
        };

        let mut total_score = Decimal::ZERO;
        let mut total_weight = Decimal::ZERO;

        let contributions = [
            &breakdown.win_rate,
            &breakdown.profit_factor,
            &breakdown.expectancy,
            &breakdown.average_rr,
        ];

        for c in contributions {
            total_score += c.score * c.weight;
            total_weight += c.weight;
        }

        let final_score = if total_weight.is_zero() {
            Decimal::new(100, 0)
        } else {
            (total_score / total_weight).round_dp(2)
        };

        let state = Self::determine_state(final_score);

        Self {
            state,
            current_score: final_score,
            breakdown,
            last_updated: timestamp,
            version: 1,
        }
    }
}
