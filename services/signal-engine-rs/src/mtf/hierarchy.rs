//! Timeframe hierarchy management

/// Hierarchical timeframe structure
#[derive(Debug, Clone)]
pub struct TimeframeHierarchy {
    /// Ordered list from highest to lowest
    timeframes: Vec<String>,
}

impl TimeframeHierarchy {
    /// Create a new hierarchy from timeframe list
    pub fn new(timeframes: &[String]) -> Self {
        let mut sorted = timeframes.to_vec();

        // Sort by timeframe magnitude (highest first)
        sorted.sort_by(|a, b| {
            let a_rank = Self::timeframe_rank(a);
            let b_rank = Self::timeframe_rank(b);
            b_rank.cmp(&a_rank) // Reverse for descending
        });

        Self { timeframes: sorted }
    }

    /// Get the ordered timeframes
    pub fn ordered(&self) -> &[String] {
        &self.timeframes
    }

    /// Get the highest timeframe
    pub fn highest(&self) -> Option<&String> {
        self.timeframes.first()
    }

    /// Get the lowest timeframe
    pub fn lowest(&self) -> Option<&String> {
        self.timeframes.last()
    }

    /// Get weight for a timeframe based on position
    pub fn weight(&self, timeframe: &str) -> f64 {
        match self.timeframes.iter().position(|t| t == timeframe) {
            Some(0) => 0.40, // Highest - structure context
            Some(1) => 0.30, // Second - bias
            Some(2) => 0.20, // Third - context
            Some(3) => 0.10, // Lowest - execution trigger
            _ => 0.0,
        }
    }

    /// Get the rank/magnitude of a timeframe
    fn timeframe_rank(tf: &str) -> u32 {
        match tf {
            "Monthly" => 1000,
            "Weekly" => 900,
            "Daily" => 800,
            "H4" | "4H" => 700,
            "H1" | "1H" => 600,
            "M30" | "30M" => 500,
            "M15" | "15M" => 400,
            "M5" | "5M" => 300,
            "M1" | "1M" => 200,
            _ => 0,
        }
    }

    /// Get standard 4-timeframe hierarchy
    pub fn standard() -> Self {
        Self::new(&[
            "H4".to_string(),
            "H1".to_string(),
            "M30".to_string(),
            "M15".to_string(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_ordering() {
        let hierarchy = TimeframeHierarchy::new(&[
            "M15".to_string(),
            "H4".to_string(),
            "M30".to_string(),
            "H1".to_string(),
        ]);

        let ordered = hierarchy.ordered();
        assert_eq!(ordered[0], "H4");
        assert_eq!(ordered[1], "H1");
        assert_eq!(ordered[2], "M30");
        assert_eq!(ordered[3], "M15");
    }

    #[test]
    fn test_weights() {
        let hierarchy = TimeframeHierarchy::standard();

        assert_eq!(hierarchy.weight("H4"), 0.40);
        assert_eq!(hierarchy.weight("H1"), 0.30);
        assert_eq!(hierarchy.weight("M30"), 0.20);
        assert_eq!(hierarchy.weight("M15"), 0.10);
    }
}
