use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscoveryState {
    Emerging,
    Stable,
    Weakening,
    Collapsing,
}

#[derive(Debug, Clone)]
pub struct EdgeDiscovery {
    pub improving_symbols: Vec<String>,
    pub degrading_symbols: Vec<String>,
    pub improving_regimes: Vec<String>,
    pub degrading_regimes: Vec<String>,
    pub improving_sessions: Vec<String>,
    pub degrading_sessions: Vec<String>,
    pub improving_patterns: Vec<String>,
    pub degrading_patterns: Vec<String>,
}

impl EdgeDiscovery {
    pub fn new() -> Self {
        Self {
            improving_symbols: Vec::new(),
            degrading_symbols: Vec::new(),
            improving_regimes: Vec::new(),
            degrading_regimes: Vec::new(),
            improving_sessions: Vec::new(),
            degrading_sessions: Vec::new(),
            improving_patterns: Vec::new(),
            degrading_patterns: Vec::new(),
        }
    }

    /// Evaluates if an entity is improving or degrading based on short vs long term expectancy
    /// Expects absolute determinism
    pub fn evaluate_state(short_term_expectancy: Decimal, long_term_expectancy: Decimal, sample_size: u32, min_sample: u32) -> Option<DiscoveryState> {
        if sample_size < min_sample {
            return None;
        }

        // If short term is better than long term by a margin, it's emerging/improving
        if short_term_expectancy > long_term_expectancy {
            Some(DiscoveryState::Emerging)
        } else if short_term_expectancy < long_term_expectancy {
            Some(DiscoveryState::Weakening)
        } else {
            Some(DiscoveryState::Stable)
        }
    }
}
