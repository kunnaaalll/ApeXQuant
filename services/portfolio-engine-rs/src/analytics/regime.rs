// src/analytics/regime.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RegimePerformanceMetrics {
    pub profit_factor: Decimal,
    pub expectancy: Decimal,
    pub win_rate: Decimal,
    pub total_trades: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegimePerformanceProfile {
    pub trending: RegimePerformanceMetrics,
    pub ranging: RegimePerformanceMetrics,
    pub high_volatility: RegimePerformanceMetrics,
    pub low_volatility: RegimePerformanceMetrics,
    pub london_session: RegimePerformanceMetrics,
    pub new_york_session: RegimePerformanceMetrics,
    pub asia_session: RegimePerformanceMetrics,
}

impl Default for RegimePerformanceProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl RegimePerformanceProfile {
    pub fn new() -> Self {
        Self {
            trending: RegimePerformanceMetrics::default(),
            ranging: RegimePerformanceMetrics::default(),
            high_volatility: RegimePerformanceMetrics::default(),
            low_volatility: RegimePerformanceMetrics::default(),
            london_session: RegimePerformanceMetrics::default(),
            new_york_session: RegimePerformanceMetrics::default(),
            asia_session: RegimePerformanceMetrics::default(),
        }
    }

    /// Determines where the portfolio performs best by returning the name of the regime
    /// with the highest expectancy that has a statistically significant trade count (e.g., > 30).
    pub fn best_performing_regime(&self) -> &'static str {
        let mut best_name = "None";
        let mut best_expectancy = Decimal::MIN;

        let mut check = |name: &'static str, metrics: &RegimePerformanceMetrics| {
            if metrics.total_trades > 30 && metrics.expectancy > best_expectancy {
                best_expectancy = metrics.expectancy;
                best_name = name;
            }
        };

        check("Trending", &self.trending);
        check("Ranging", &self.ranging);
        check("High Volatility", &self.high_volatility);
        check("Low Volatility", &self.low_volatility);
        check("London Session", &self.london_session);
        check("New York Session", &self.new_york_session);
        check("Asia Session", &self.asia_session);

        best_name
    }
}
