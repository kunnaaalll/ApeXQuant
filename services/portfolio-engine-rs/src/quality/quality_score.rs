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
    pub weight: f64,
    pub score: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    pub timestamp: u64,
    pub version: u64,
    pub state: PortfolioQualityState,
    pub composite_score: f64,
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
    pub current_score: f64, // 0.0 to 100.0
    pub breakdown: PortfolioQualityBreakdown,
    pub last_updated: u64,
    pub version: u64,
}

impl PortfolioQuality {
    pub fn new(timestamp: u64) -> Self {
        let default_contribution = QualityContribution {
            weight: 0.0,
            score: 0.0,
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
            current_score: 50.0,
            breakdown,
            last_updated: timestamp,
            version: 1,
        }
    }

    pub fn determine_state(score: f64) -> PortfolioQualityState {
        if score >= 90.0 {
            PortfolioQualityState::Excellent
        } else if score >= 75.0 {
            PortfolioQualityState::Good
        } else if score >= 50.0 {
            PortfolioQualityState::Neutral
        } else if score >= 25.0 {
            PortfolioQualityState::Weak
        } else {
            PortfolioQualityState::Critical
        }
    }

    pub fn apply_event(&mut self, _event: QualityEvent, new_score: f64, new_breakdown: PortfolioQualityBreakdown, timestamp: u64) -> QualitySnapshot {
        // Enforce invariants
        let bounded_score = new_score.clamp(0.0, 100.0);
        
        self.current_score = bounded_score;
        self.state = Self::determine_state(bounded_score);
        self.breakdown = new_breakdown;
        self.last_updated = timestamp;
        self.version += 1;

        self.create_snapshot()
    }

    pub fn apply_decay(&mut self, decay_factor: f64, timestamp: u64) -> QualitySnapshot {
        let decayed_score = (self.current_score - decay_factor).max(0.0);
        let mut new_breakdown = self.breakdown.clone();
        
        // Update a specific component to reflect decay, here we apply it generally.
        new_breakdown.recent_performance = QualityContribution {
            weight: self.breakdown.recent_performance.weight,
            score: (self.breakdown.recent_performance.score - decay_factor).max(0.0),
            reason: "Time decay applied due to inactivity or sustained poor performance".to_string(),
        };

        self.apply_event(QualityEvent::DecayApplied, decayed_score, new_breakdown, timestamp)
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
}
