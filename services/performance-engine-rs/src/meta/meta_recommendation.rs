use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Meta Recommendation Engine
//
// Produces one of five actions per strategy, with full explanation.
// No AI. No randomness. Evidence-based. Deterministic. Auditable.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetaAction {
    Continue,           // Strategy is performing — maintain current allocation.
    IncreaseAllocation, // Strategy is outperforming — increase exposure.
    Reduce,             // Strategy is weakening — reduce exposure.
    Pause,              // Strategy needs observation — halt new entries.
    Retire,             // Strategy has lost edge — permanently decommission.
    Research,           // Strategy requires investigation before any action.
}

impl std::fmt::Display for MetaAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaAction::Continue => write!(f, "Continue"),
            MetaAction::IncreaseAllocation => write!(f, "IncreaseAllocation"),
            MetaAction::Reduce => write!(f, "Reduce"),
            MetaAction::Pause => write!(f, "Pause"),
            MetaAction::Retire => write!(f, "Retire"),
            MetaAction::Research => write!(f, "Research"),
        }
    }
}

/// Full evidence-based recommendation with mandatory explanation chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaRecommendation {
    pub action: MetaAction,
    pub reason: String,
    pub largest_contributor: String, // the single metric most responsible for this decision
    pub largest_weakness: String,    // the single metric most at risk
    pub confidence: Decimal,         // [0, 1]
    pub historical_evidence: u32,    // number of trades backing this recommendation
}

/// Inputs required for the engine — all derived from previously computed analytics.
#[derive(Debug, Clone)]
pub struct MetaRecommendationInput {
    pub strategy_name: String,
    pub health_score: u8, // 0–100 from StrategyHealth
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub win_rate: Decimal,
    pub max_drawdown: Decimal,
    pub confidence: Decimal,
    pub stability: Decimal,
    pub trade_count: u32,
    /// Fraction of expectancy decline over last N periods (positive = declined)
    pub expectancy_drift: Decimal,
    /// OOS / IS performance ratio (None if no OOS data)
    pub oos_ratio: Option<Decimal>,
    /// Overfit confidence penalty [0, 1] — 1 = no penalty
    pub overfit_penalty: Decimal,
}

pub struct MetaRecommendationEngine;

impl MetaRecommendationEngine {
    /// Deterministic decision tree — identical inputs always produce identical output.
    pub fn recommend(input: &MetaRecommendationInput) -> MetaRecommendation {
        // ── Gate 1: Retire — irreversible collapse ────────────────────────────
        if input.health_score == 0
            || input.expectancy < dec!(-0.15)
            || input.profit_factor < dec!(0.65)
        {
            return MetaRecommendation {
                action: MetaAction::Retire,
                reason: format!(
                    "{} has suffered a terminal edge collapse. Expectancy: {:.3}R, PF: {:.3}, health: {}.",
                    input.strategy_name, input.expectancy, input.profit_factor, input.health_score
                ),
                largest_contributor: "Edge collapse (expectancy / PF breach)".to_string(),
                largest_weakness: "Expectancy".to_string(),
                confidence: input.confidence,
                historical_evidence: input.trade_count,
            };
        }

        // ── Gate 2: Pause — instability or overfitting risk ──────────────────
        if input.health_score < 20
            || input.max_drawdown > dec!(0.20)
            || input.overfit_penalty < dec!(0.50)
        {
            return MetaRecommendation {
                action: MetaAction::Pause,
                reason: format!(
                    "{} flagged for pause. Health: {}, drawdown: {:.1}%, overfit penalty: {:.2}.",
                    input.strategy_name,
                    input.health_score,
                    input.max_drawdown * dec!(100),
                    input.overfit_penalty
                ),
                largest_contributor: if input.overfit_penalty < dec!(0.50) {
                    "Overfitting risk".to_string()
                } else {
                    "Drawdown / health breach".to_string()
                },
                largest_weakness: "Overfit penalty".to_string(),
                confidence: input.confidence * input.overfit_penalty,
                historical_evidence: input.trade_count,
            };
        }

        // ── Gate 3: Reduce — drifting edge ───────────────────────────────────
        if input.expectancy_drift > dec!(0.08)        // expectancy declined > 8%
            || input.health_score < 40
            || input.profit_factor < dec!(1.20)
        {
            return MetaRecommendation {
                action: MetaAction::Reduce,
                reason: format!(
                    "{} is weakening. Expectancy drift: {:.1}%, PF: {:.3}, health: {}.",
                    input.strategy_name,
                    input.expectancy_drift * dec!(100),
                    input.profit_factor,
                    input.health_score
                ),
                largest_contributor: "Expectancy / profit factor degradation".to_string(),
                largest_weakness: "Profit factor".to_string(),
                confidence: input.confidence * dec!(0.8),
                historical_evidence: input.trade_count,
            };
        }

        // ── Gate 4: Research — insufficient sample or ambiguous signal ────────
        if input.trade_count < 50
            || input.confidence < dec!(0.40)
            || input.oos_ratio.is_some_and(|r| r < dec!(0.70))
        {
            return MetaRecommendation {
                action: MetaAction::Research,
                reason: format!(
                    "{} requires further investigation. Trades: {}, confidence: {:.2}, OOS ratio: {}.",
                    input.strategy_name,
                    input.trade_count,
                    input.confidence,
                    input.oos_ratio
                        .map(|r| format!("{:.2}", r))
                        .unwrap_or_else(|| "n/a".to_string())
                ),
                largest_contributor: "Insufficient sample quality".to_string(),
                largest_weakness: "Confidence".to_string(),
                confidence: input.confidence,
                historical_evidence: input.trade_count,
            };
        }

        // ── Gate 5: Increase — elite performance ─────────────────────────────
        if input.health_score >= 80
            && input.expectancy > dec!(0.15)
            && input.profit_factor >= dec!(2.0)
            && input.stability >= dec!(0.75)
            && input.expectancy_drift <= dec!(0)   // not drifting negative
            && input.overfit_penalty >= dec!(0.85)
        {
            return MetaRecommendation {
                action: MetaAction::IncreaseAllocation,
                reason: format!(
                    "{} is elite. Health: {}, expectancy: {:.3}R, PF: {:.3}, stability: {:.2}.",
                    input.strategy_name,
                    input.health_score,
                    input.expectancy,
                    input.profit_factor,
                    input.stability
                ),
                largest_contributor: "Strong expectancy and stability".to_string(),
                largest_weakness: "None identified".to_string(),
                confidence: input.confidence,
                historical_evidence: input.trade_count,
            };
        }

        // ── Default: Continue ─────────────────────────────────────────────────
        MetaRecommendation {
            action: MetaAction::Continue,
            reason: format!(
                "{} is performing within acceptable bounds. Health: {}, expectancy: {:.3}R, PF: {:.3}.",
                input.strategy_name, input.health_score, input.expectancy, input.profit_factor
            ),
            largest_contributor: format!(
                "Expectancy {:.3}R above zero",
                input.expectancy
            ),
            largest_weakness: if input.max_drawdown > dec!(0.10) {
                format!("Drawdown {:.1}%", input.max_drawdown * dec!(100))
            } else {
                "None significant".to_string()
            },
            confidence: input.confidence,
            historical_evidence: input.trade_count,
        }
    }
}
