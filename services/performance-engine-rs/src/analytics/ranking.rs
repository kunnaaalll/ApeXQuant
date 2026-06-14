
use serde::{Deserialize, Serialize};

use crate::regime::models::RegimeAssessment;
use crate::session::models::SessionAssessment;
use crate::symbol::models::SymbolAssessment;
use crate::timeframe::models::TimeframeAssessment;
use super::adequacy::AdequacyScore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextRankingResult {
    pub top_regimes: Vec<RegimeAssessment>,
    pub bottom_regimes: Vec<RegimeAssessment>,
    pub top_sessions: Vec<SessionAssessment>,
    pub bottom_sessions: Vec<SessionAssessment>,
    pub top_symbols: Vec<SymbolAssessment>,
    pub bottom_symbols: Vec<SymbolAssessment>,
    pub top_timeframes: Vec<TimeframeAssessment>,
    pub bottom_timeframes: Vec<TimeframeAssessment>,
    pub overall_adequacy: AdequacyScore,
}

pub struct ContextRanking;

impl ContextRanking {
    pub fn rank(
        regimes: Vec<RegimeAssessment>,
        sessions: Vec<SessionAssessment>,
        symbols: Vec<SymbolAssessment>,
        timeframes: Vec<TimeframeAssessment>,
        overall_adequacy: AdequacyScore,
    ) -> ContextRankingResult {
        let mut sorted_regimes = regimes;
        sorted_regimes.sort_by(|a, b| b.expectancy.cmp(&a.expectancy));
        let (top_regimes, bottom_regimes) = Self::split_top_bottom(sorted_regimes);

        let mut sorted_sessions = sessions;
        sorted_sessions.sort_by(|a, b| b.expectancy.cmp(&a.expectancy));
        let (top_sessions, bottom_sessions) = Self::split_top_bottom(sorted_sessions);

        let mut sorted_symbols = symbols;
        sorted_symbols.sort_by(|a, b| b.expectancy.cmp(&a.expectancy));
        let (top_symbols, bottom_symbols) = Self::split_top_bottom(sorted_symbols);

        let mut sorted_timeframes = timeframes;
        sorted_timeframes.sort_by(|a, b| b.expectancy.cmp(&a.expectancy));
        let (top_timeframes, bottom_timeframes) = Self::split_top_bottom(sorted_timeframes);

        ContextRankingResult {
            top_regimes,
            bottom_regimes,
            top_sessions,
            bottom_sessions,
            top_symbols,
            bottom_symbols,
            top_timeframes,
            bottom_timeframes,
            overall_adequacy,
        }
    }

    fn split_top_bottom<T: Clone>(sorted_items: Vec<T>) -> (Vec<T>, Vec<T>) {
        if sorted_items.is_empty() {
            return (Vec::new(), Vec::new());
        }
        
        // Take top 3 for top, bottom 3 for bottom (or less if not enough items)
        let len = sorted_items.len();
        let top_count = std::cmp::min(3, len);
        let top = sorted_items[..top_count].to_vec();
        
        let bottom_start = if len > top_count * 2 { len - top_count } else { top_count };
        let mut bottom = sorted_items[bottom_start..].to_vec();
        bottom.reverse(); // Lowest expectancy first
        
        (top, bottom)
    }
}
