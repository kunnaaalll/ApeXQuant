use super::circuit_breaker::ExecutionProtectionState;
use super::failure_tracker::FailureTracker;
use super::fill_quality_guards::FillQualityGuards;
use super::latency_guards::LatencyGuards;
use super::liquidity_guards::LiquidityGuards;
use super::slippage_guards::SlippageGuards;
use super::spread_guards::SpreadGuards;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeGuardAction {
    Allow,
    ReduceSize,
    Delay,
    SplitOrder,
    Block,
    FreezeTrading,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeGuards {
    pub current_protection_state: ExecutionProtectionState,
    pub spread: SpreadGuards,
    pub liquidity: LiquidityGuards,
    pub latency: LatencyGuards,
    pub slippage: SlippageGuards,
    pub fill_quality: FillQualityGuards,
    pub failures: FailureTracker,
}

impl TradeGuards {
    pub fn evaluate(&self) -> TradeGuardAction {
        if self.current_protection_state == ExecutionProtectionState::Frozen {
            return TradeGuardAction::FreezeTrading;
        }

        if self.current_protection_state == ExecutionProtectionState::Critical {
            return TradeGuardAction::Block;
        }

        let failure_score = self.failures.get_score();
        if failure_score >= 80 {
            return TradeGuardAction::FreezeTrading;
        } else if failure_score >= 50 {
            return TradeGuardAction::Block;
        }

        let latency_score = self.latency.get_score();
        if latency_score >= 50 {
            return TradeGuardAction::Delay;
        }

        let liquidity_regime = self.liquidity.get_regime();
        if liquidity_regime == super::liquidity_guards::LiquidityRegime::Poor {
            return TradeGuardAction::SplitOrder;
        } else if liquidity_regime == super::liquidity_guards::LiquidityRegime::Broken {
            return TradeGuardAction::Block;
        }

        let spread_score = self.spread.get_score();
        if spread_score >= 50 {
            return TradeGuardAction::ReduceSize;
        }

        let slippage_score = self.slippage.get_penalty_score();
        if slippage_score >= 80 {
            return TradeGuardAction::Block;
        } else if slippage_score >= 40 {
            return TradeGuardAction::ReduceSize;
        }

        TradeGuardAction::Allow
    }
}
