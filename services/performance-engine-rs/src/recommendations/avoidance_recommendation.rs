use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvoidanceState {
    Warning,
    Avoid,
    Forbidden,
}

#[derive(Debug, Clone)]
pub struct AvoidanceEntry {
    pub name: String,
    pub state: AvoidanceState,
    pub penalty_score: Decimal,
}

#[derive(Debug, Clone)]
pub struct AvoidanceRecommendation {
    pub worst_symbols: Vec<AvoidanceEntry>,
    pub worst_sessions: Vec<AvoidanceEntry>,
    pub worst_regimes: Vec<AvoidanceEntry>,
    pub worst_patterns: Vec<AvoidanceEntry>,
    pub worst_timeframes: Vec<AvoidanceEntry>,
}

#[derive(Debug, Clone)]
pub struct AvoidanceEngine {
    pub warning_threshold: Decimal,
    pub avoid_threshold: Decimal,
    pub forbidden_threshold: Decimal,
}

impl AvoidanceEngine {
    pub fn new(warning: Decimal, avoid: Decimal, forbidden: Decimal) -> Self {
        Self {
            warning_threshold: warning,
            avoid_threshold: avoid,
            forbidden_threshold: forbidden,
        }
    }

    pub fn evaluate_entity(&self, name: String, penalty_score: Decimal) -> Option<AvoidanceEntry> {
        let state = if penalty_score >= self.forbidden_threshold {
            AvoidanceState::Forbidden
        } else if penalty_score >= self.avoid_threshold {
            AvoidanceState::Avoid
        } else if penalty_score >= self.warning_threshold {
            AvoidanceState::Warning
        } else {
            return None;
        };

        Some(AvoidanceEntry {
            name,
            state,
            penalty_score,
        })
    }
}
