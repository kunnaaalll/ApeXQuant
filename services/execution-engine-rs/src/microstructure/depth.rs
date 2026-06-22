use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthLevel {
    Excellent,
    Strong,
    Normal,
    Weak,
    Thin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBookDepth {
    pub level1_vol: Decimal,
    pub level2_vol: Decimal,
    pub level3_vol: Decimal,
    pub cumulative_depth: Decimal,
}

impl OrderBookDepth {
    pub fn new(level1: Decimal, level2: Decimal, level3: Decimal) -> Result<Self, &'static str> {
        if level1 < Decimal::ZERO || level2 < Decimal::ZERO || level3 < Decimal::ZERO {
            return Err("Depth volumes cannot be negative");
        }
        let cumulative = level1 + level2 + level3;
        Ok(Self {
            level1_vol: level1,
            level2_vol: level2,
            level3_vol: level3,
            cumulative_depth: cumulative,
        })
    }

    pub fn grade(&self) -> DepthLevel {
        // A simple grading heuristic for the test and deterministic behavior.
        // We will classify based on cumulative_depth arbitrarily.
        if self.cumulative_depth >= Decimal::new(1000, 0) {
            DepthLevel::Excellent
        } else if self.cumulative_depth >= Decimal::new(500, 0) {
            DepthLevel::Strong
        } else if self.cumulative_depth >= Decimal::new(100, 0) {
            DepthLevel::Normal
        } else if self.cumulative_depth >= Decimal::new(10, 0) {
            DepthLevel::Weak
        } else {
            DepthLevel::Thin
        }
    }
}
