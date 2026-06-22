use rust_decimal::Decimal;
use crate::execution_cost::spread_cost::SpreadCost;
use crate::execution_cost::slippage_cost::SlippageCost;
use crate::execution_cost::impact_cost::ImpactCost;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TotalExecutionCostGrade {
    Excellent,
    Good,
    Average,
    Poor,
    Extreme,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotalExecutionCost {
    pub total_usd: Decimal,
    pub grade: TotalExecutionCostGrade,
}

impl TotalExecutionCost {
    pub fn calculate(
        spread_cost: &SpreadCost,
        slippage_cost: &SlippageCost,
        impact_cost: &ImpactCost,
        notional_usd: Decimal,
    ) -> Result<Self, &'static str> {
        if notional_usd <= Decimal::ZERO {
            return Err("Notional must be greater than zero");
        }

        let total_usd = spread_cost.cost_usd + slippage_cost.cost_usd + impact_cost.cost_usd;
        
        let bps_cost = (total_usd / notional_usd) * Decimal::new(10000, 0);
        use rust_decimal::prelude::ToPrimitive;
        let bps_cost_u64 = bps_cost.to_u64().unwrap_or(100);

        let grade = if bps_cost_u64 <= 1 {
            TotalExecutionCostGrade::Excellent
        } else if bps_cost_u64 <= 5 {
            TotalExecutionCostGrade::Good
        } else if bps_cost_u64 <= 15 {
            TotalExecutionCostGrade::Average
        } else if bps_cost_u64 <= 30 {
            TotalExecutionCostGrade::Poor
        } else {
            TotalExecutionCostGrade::Extreme
        };

        Ok(Self { total_usd, grade })
    }
}
