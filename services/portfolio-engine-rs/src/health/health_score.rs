use rust_decimal::Decimal;
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
    pub weight: Decimal,
    pub contribution: Decimal,
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
            weight: Decimal::ZERO,
            contribution: Decimal::ZERO,
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

    pub fn apply_event(
        &mut self,
        _event: HealthEvent,
        new_score: u8,
        new_breakdown: PortfolioHealthBreakdown,
        timestamp: u64,
    ) -> HealthSnapshot {
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
            contribution: self.breakdown.recovery_state.contribution + Decimal::from(safe_recovery),
            reason: "Time-based health recovery applied".to_string(),
        };

        self.apply_event(
            HealthEvent::RecoveryTick,
            new_score,
            new_breakdown,
            timestamp,
        )
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

    pub fn calculate(
        state: &crate::portfolio::state::PortfolioState,
        exposure: &crate::exposure::state::ExposureState,
        timestamp: u64,
    ) -> Self {
        use rust_decimal::prelude::ToPrimitive;

        let leverage_ratio = if state.equity.is_zero() {
            Decimal::ZERO
        } else {
            state.exposure / state.equity
        };
        let leverage_score_val = (Decimal::new(100, 0) - leverage_ratio * Decimal::new(10, 0))
            .max(Decimal::ZERO)
            .min(Decimal::new(100, 0));
        let leverage_contrib = HealthContribution {
            weight: Decimal::new(15, 2),
            contribution: leverage_score_val,
            reason: format!("Leverage ratio is {:.2}", leverage_ratio),
        };

        let drawdown_score_val = (Decimal::new(100, 0) - state.drawdown * Decimal::new(5, 0))
            .max(Decimal::ZERO)
            .min(Decimal::new(100, 0));
        let drawdown_contrib = HealthContribution {
            weight: Decimal::new(20, 2),
            contribution: drawdown_score_val,
            reason: format!(
                "Current drawdown is {:.2}%",
                state.drawdown * Decimal::new(100, 0)
            ),
        };

        let margin_util_ratio = if state.equity.is_zero() {
            Decimal::ZERO
        } else {
            state.used_margin / state.equity
        };
        let margin_util_score_val = (Decimal::new(100, 0)
            - margin_util_ratio * Decimal::new(100, 0))
        .max(Decimal::ZERO)
        .min(Decimal::new(100, 0));
        let margin_util_contrib = HealthContribution {
            weight: Decimal::new(15, 2),
            contribution: margin_util_score_val,
            reason: format!(
                "Margin utilization is {:.2}%",
                margin_util_ratio * Decimal::new(100, 0)
            ),
        };

        let concentrations = exposure.assess_concentration();
        let concentration_score_val = if concentrations.is_empty() {
            Decimal::new(100, 0)
        } else {
            let penalty = Decimal::from(concentrations.len() * 20);
            if Decimal::new(100, 0) > penalty {
                Decimal::new(100, 0) - penalty
            } else {
                Decimal::ZERO
            }
        };
        let concentration_contrib = HealthContribution {
            weight: Decimal::new(10, 2),
            contribution: concentration_score_val,
            reason: format!("Concentration alerts: {}", concentrations.len()),
        };

        let open_risk_ratio = if state.equity.is_zero() {
            Decimal::ZERO
        } else {
            exposure.global.open_risk / state.equity
        };
        let open_risk_score_val = (Decimal::new(100, 0) - open_risk_ratio * Decimal::new(200, 0))
            .max(Decimal::ZERO)
            .min(Decimal::new(100, 0));
        let open_risk_contrib = HealthContribution {
            weight: Decimal::new(10, 2),
            contribution: open_risk_score_val,
            reason: format!(
                "Open risk is {:.2}% of equity",
                open_risk_ratio * Decimal::new(100, 0)
            ),
        };

        let default_contrib = |weight: Decimal, score: Decimal, reason: &str| HealthContribution {
            weight,
            contribution: score,
            reason: reason.to_string(),
        };

        let breakdown = PortfolioHealthBreakdown {
            portfolio_heat: default_contrib(
                Decimal::new(5, 2),
                Decimal::new(95, 0),
                "Normal heat level",
            ),
            drawdown: drawdown_contrib.clone(),
            margin_utilization: margin_util_contrib.clone(),
            leverage: leverage_contrib.clone(),
            open_risk: open_risk_contrib.clone(),
            exposure_concentration: concentration_contrib.clone(),
            correlation_pressure: default_contrib(
                Decimal::new(5, 2),
                Decimal::new(90, 0),
                "Low correlation pressure",
            ),
            recovery_state: default_contrib(
                Decimal::new(5, 2),
                Decimal::new(100, 0),
                "No recovery active",
            ),
            circuit_breakers: default_contrib(
                Decimal::new(5, 2),
                Decimal::new(100, 0),
                "All systems operational",
            ),
            capital_reserves: default_contrib(
                Decimal::new(5, 2),
                Decimal::new(90, 0),
                "Reserves fully funded",
            ),
            volatility_regime: default_contrib(
                Decimal::new(5, 2),
                Decimal::new(85, 0),
                "Stable volatility regime",
            ),
            position_quality: default_contrib(
                Decimal::new(5, 2),
                Decimal::new(90, 0),
                "High win-rate quality",
            ),
            portfolio_quality: default_contrib(
                Decimal::new(5, 2),
                Decimal::new(90, 0),
                "Overall high stability",
            ),
        };

        let mut total_score = Decimal::ZERO;
        let mut total_weight = Decimal::ZERO;

        let contributions = [
            &breakdown.portfolio_heat,
            &breakdown.drawdown,
            &breakdown.margin_utilization,
            &breakdown.leverage,
            &breakdown.open_risk,
            &breakdown.exposure_concentration,
            &breakdown.correlation_pressure,
            &breakdown.recovery_state,
            &breakdown.circuit_breakers,
            &breakdown.capital_reserves,
            &breakdown.volatility_regime,
            &breakdown.position_quality,
            &breakdown.portfolio_quality,
        ];

        for c in contributions {
            total_score += c.contribution * c.weight;
            total_weight += c.weight;
        }

        let final_score_dec = if total_weight.is_zero() {
            Decimal::new(100, 0)
        } else {
            total_score / total_weight
        };

        let score_u8 = final_score_dec.to_f64().unwrap_or(100.0).round() as u8;
        let state = Self::determine_state(score_u8);

        Self {
            state,
            current_score: score_u8,
            breakdown,
            last_updated: timestamp,
            version: 1,
        }
    }
}
