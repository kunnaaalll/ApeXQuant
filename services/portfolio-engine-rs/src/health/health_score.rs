use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PortfolioHealthState {
    Critical,
    Weak,
    Normal,
    Healthy,
    Excellent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHealthBreakdown {
    pub portfolio_heat: HealthContribution,
    pub drawdown: HealthContribution,
    pub margin_utilization: HealthContribution,
    pub leverage: HealthContribution,
    pub open_risk: HealthContribution,
    pub exposure_concentration: HealthContribution,
    pub correlation_pressure: HealthContribution,
    pub recovery_state: HealthContribution,
    pub circuit_breakers: HealthContribution,
    pub capital_reserves: HealthContribution,
    pub volatility_regime: HealthContribution,
    pub position_quality: HealthContribution,
    pub portfolio_quality: HealthContribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthContribution {
    pub weight: f64,
    pub contribution: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub timestamp: u64,
    pub version: u64,
    pub state: PortfolioHealthState,
    pub composite_score: u8,
    pub breakdown: PortfolioHealthBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthEvent {
    PositionChanged,
    PnLChanged,
    HeatChanged,
    AllocationChanged,
    RecoveryChanged,
    CircuitBreaker,
    VolatilityChanged,
    RecoveryTick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHealth {
    pub state: PortfolioHealthState,
    pub current_score: u8, // 0-100
    pub breakdown: PortfolioHealthBreakdown,
    pub last_updated: u64,
    pub version: u64,
}

impl PortfolioHealth {
    pub fn new(timestamp: u64) -> Self {
        let default_contribution = HealthContribution {
            weight: 0.0,
            contribution: 0.0,
            reason: "Initialization".to_string(),
        };
        let breakdown = PortfolioHealthBreakdown {
            portfolio_heat: default_contribution.clone(),
            drawdown: default_contribution.clone(),
            margin_utilization: default_contribution.clone(),
            leverage: default_contribution.clone(),
            open_risk: default_contribution.clone(),
            exposure_concentration: default_contribution.clone(),
            correlation_pressure: default_contribution.clone(),
            recovery_state: default_contribution.clone(),
            circuit_breakers: default_contribution.clone(),
            capital_reserves: default_contribution.clone(),
            volatility_regime: default_contribution.clone(),
            position_quality: default_contribution.clone(),
            portfolio_quality: default_contribution,
        };

        Self {
            state: PortfolioHealthState::Normal,
            current_score: 50,
            breakdown,
            last_updated: timestamp,
            version: 1,
        }
    }

    pub fn determine_state(score: u8) -> PortfolioHealthState {
        if score >= 90 {
            PortfolioHealthState::Excellent
        } else if score >= 75 {
            PortfolioHealthState::Healthy
        } else if score >= 50 {
            PortfolioHealthState::Normal
        } else if score >= 25 {
            PortfolioHealthState::Weak
        } else {
            PortfolioHealthState::Critical
        }
    }

    pub fn apply_event(&mut self, _event: HealthEvent, new_score: u8, new_breakdown: PortfolioHealthBreakdown, timestamp: u64) -> HealthSnapshot {
        // Enforce invariants: score must be between 0 and 100.
        // It is inherently bounded since it's u8, but we clamp to max 100.
        let bounded_score = new_score.min(100);
        
        self.current_score = bounded_score;
        self.state = Self::determine_state(bounded_score);
        self.breakdown = new_breakdown;
        self.last_updated = timestamp;
        self.version += 1;

        self.create_snapshot()
    }

    pub fn apply_recovery(&mut self, recovery_points: u8, timestamp: u64) -> HealthSnapshot {
        // Enforce gradual recovery. Cannot jump from Critical to Healthy instantly.
        // Limit recovery tick impact.
        let safe_recovery = recovery_points.min(5); // e.g. max 5 points per tick
        let new_score = self.current_score.saturating_add(safe_recovery).min(100);
        
        let mut new_breakdown = self.breakdown.clone();
        new_breakdown.recovery_state = HealthContribution {
            weight: self.breakdown.recovery_state.weight,
            contribution: self.breakdown.recovery_state.contribution + (safe_recovery as f64),
            reason: "Time-based health recovery applied".to_string(),
        };

        self.apply_event(HealthEvent::RecoveryTick, new_score, new_breakdown, timestamp)
    }

    pub fn create_snapshot(&self) -> HealthSnapshot {
        HealthSnapshot {
            timestamp: self.last_updated,
            version: self.version,
            state: self.state,
            composite_score: self.current_score,
            breakdown: self.breakdown.clone(),
        }
    }
}
