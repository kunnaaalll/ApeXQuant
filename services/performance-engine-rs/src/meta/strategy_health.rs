use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt;

// ─────────────────────────────────────────────────────────────────────────────
// StrategyHealth
// Range : 0–100
// States: Excellent (90–100) | Healthy (70–89) | Normal (50–69)
//         | Weak (30–49) | Critical (10–29) | Dead (0–9)
//
// Invariants
//   • Collapse is IMMEDIATE — score can drop to 0 in one step.
//   • Recovery is GRADUAL   — score can rise by at most MAX_RECOVERY_STEP per step.
//   • Score is always clamped to [0, 100].
// ─────────────────────────────────────────────────────────────────────────────

const MAX_RECOVERY_STEP: u8 = 5; // maximum health points recoverable per evaluation cycle

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthState {
    Excellent,
    Healthy,
    Normal,
    Weak,
    Critical,
    Dead,
}

impl fmt::Display for HealthState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthState::Excellent => write!(f, "Excellent"),
            HealthState::Healthy => write!(f, "Healthy"),
            HealthState::Normal => write!(f, "Normal"),
            HealthState::Weak => write!(f, "Weak"),
            HealthState::Critical => write!(f, "Critical"),
            HealthState::Dead => write!(f, "Dead"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyHealth {
    /// 0–100 score
    pub score: u8,
    pub state: HealthState,
    pub previous_score: u8,
    pub delta: i16, // signed: positive = recovery, negative = degradation
    pub is_collapsed: bool,
    pub explanation: String,
}

impl StrategyHealth {
    pub fn from_score(score: u8) -> Self {
        let state = Self::classify(score);
        Self {
            score,
            state,
            previous_score: score,
            delta: 0,
            is_collapsed: score <= 9,
            explanation: format!("Health initialised at {} — {}", score, state),
        }
    }

    /// Compute next-period health.
    /// `raw_new_score` is the unconstrained desired score from the caller.
    /// Collapse is always immediate; recovery is capped at MAX_RECOVERY_STEP.
    pub fn transition(&self, raw_new_score: u8) -> StrategyHealth {
        let clamped = raw_new_score.min(100);
        let prev = self.score;

        // Collapse: score drops by any amount — allow fully.
        // Recovery: score can increase by at most MAX_RECOVERY_STEP per step.
        let next_score = if clamped < prev {
            // Degradation / collapse — immediate
            clamped
        } else {
            // Recovery — gradual
            let increase = clamped.saturating_sub(prev);
            prev.saturating_add(increase.min(MAX_RECOVERY_STEP))
        };

        let state = Self::classify(next_score);
        let delta = next_score as i16 - prev as i16;
        let is_collapsed = next_score <= 9;

        let explanation = format!(
            "Health transitioned {} → {} ({}). State: {}. Collapsed: {}.",
            prev,
            next_score,
            if delta >= 0 {
                format!("+{}", delta)
            } else {
                delta.to_string()
            },
            state,
            is_collapsed,
        );

        StrategyHealth {
            score: next_score,
            state,
            previous_score: prev,
            delta,
            is_collapsed,
            explanation,
        }
    }

    /// Synthesise a health score from component metrics.
    /// All inputs are normalised [0, 1] unless noted.
    pub fn synthesise(
        win_rate: Decimal,        // [0, 1]
        expectancy: Decimal,      // in R-multiples; negative = losing
        profit_factor: Decimal,   // 1.0 = break even
        max_drawdown: Decimal,    // [0, 1] positive fraction
        confidence: Decimal,      // [0, 1]
        stability: Decimal,       // [0, 1]
    ) -> u8 {
        // Immediate collapse guards
        if expectancy < dec!(-0.10) || profit_factor < dec!(0.70) {
            return 0; // Dead
        }

        let mut score = dec!(0);

        // Win rate contribution (max 20 pts)
        score += (win_rate * dec!(20)).min(dec!(20));

        // Expectancy contribution (max 25 pts) — anchored at 0.1R expectancy = 25 pts
        let expectancy_pts = if expectancy >= dec!(0.10) {
            dec!(25)
        } else if expectancy >= dec!(0) {
            expectancy / dec!(0.10) * dec!(25)
        } else {
            dec!(0)
        };
        score += expectancy_pts;

        // Profit factor contribution (max 20 pts) — PF ≥ 2.0 = full pts
        let pf_pts = if profit_factor >= dec!(2) {
            dec!(20)
        } else if profit_factor >= dec!(1) {
            (profit_factor - dec!(1)) / dec!(1) * dec!(20)
        } else {
            dec!(0)
        };
        score += pf_pts;

        // Drawdown penalty (max 15 pts lost) — 20%+ drawdown = 0 pts here
        let dd_pts = if max_drawdown <= dec!(0.05) {
            dec!(15)
        } else if max_drawdown >= dec!(0.20) {
            dec!(0)
        } else {
            (dec!(1) - (max_drawdown - dec!(0.05)) / dec!(0.15)) * dec!(15)
        };
        score += dd_pts;

        // Confidence contribution (max 10 pts)
        score += (confidence * dec!(10)).min(dec!(10));

        // Stability contribution (max 10 pts)
        score += (stability * dec!(10)).min(dec!(10));

        // Clamp to [0, 100]
        let raw = score.clamp(dec!(0), dec!(100));
        // Convert to u8 — truncate
        raw.to_string()
            .split('.')
            .next()
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0)
    }

    fn classify(score: u8) -> HealthState {
        match score {
            90..=100 => HealthState::Excellent,
            70..=89 => HealthState::Healthy,
            50..=69 => HealthState::Normal,
            30..=49 => HealthState::Weak,
            10..=29 => HealthState::Critical,
            _ => HealthState::Dead,
        }
    }
}
