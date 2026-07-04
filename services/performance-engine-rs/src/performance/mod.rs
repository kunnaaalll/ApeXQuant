use rust_decimal::Decimal;

pub struct PerformanceAttribution {
    pub allocation_return: Decimal,
    pub selection_return: Decimal,
}

impl PerformanceAttribution {
    /// Calculate attribution: returns allocation vs selection performance
    pub fn calculate(
        portfolio_return: Decimal,
        benchmark_return: Decimal,
    ) -> Self {
        Self {
            allocation_return: benchmark_return,
            selection_return: portfolio_return - benchmark_return,
        }
    }
}
