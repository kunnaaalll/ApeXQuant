use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAnalytics {
    pub best_conditions: String,
    pub worst_conditions: String,
    pub strongest_context: String,
    pub weakest_context: String,
    pub context_concentration: Decimal,
    pub degradation_hotspots: Vec<String>,
}

impl ContextAnalytics {
    pub fn new() -> Self {
        Self {
            best_conditions: String::new(),
            worst_conditions: String::new(),
            strongest_context: String::new(),
            weakest_context: String::new(),
            context_concentration: Decimal::from(0),
            degradation_hotspots: Vec::new(),
        }
    }
}

impl Default for ContextAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyAnalytics {
    pub strongest_dimension: String,
    pub weakest_dimension: String,
    pub confidence_acceleration: Decimal,
    pub degradation_acceleration: Decimal,
    pub recovery_speed: Decimal,
}

impl StrategyAnalytics {
    pub fn new() -> Self {
        Self {
            strongest_dimension: String::new(),
            weakest_dimension: String::new(),
            confidence_acceleration: Decimal::from(0),
            degradation_acceleration: Decimal::from(0),
            recovery_speed: Decimal::from(0),
        }
    }
}

impl Default for StrategyAnalytics {
    fn default() -> Self {
        Self::new()
    }
}
