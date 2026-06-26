use rust_decimal::Decimal;

pub struct SlippageContext {
    pub order_size: Decimal,
    pub available_liquidity: Decimal,
    pub is_news_event: bool,
    pub volatility: Decimal,
}

pub struct SlippageModel {
    pub base_slippage: Decimal,
    pub news_multiplier: Decimal,
}

impl SlippageModel {
    pub fn new(base_slippage: Decimal, news_multiplier: Decimal) -> Self {
        Self {
            base_slippage,
            news_multiplier,
        }
    }

    pub fn expected_slippage(&self, ctx: &SlippageContext) -> Decimal {
        let size_impact = if ctx.available_liquidity > Decimal::ZERO {
            ctx.order_size / ctx.available_liquidity
        } else {
            Decimal::ONE
        };

        let mut slippage = self.base_slippage * size_impact * ctx.volatility;
        if ctx.is_news_event {
            slippage *= self.news_multiplier;
        }
        slippage
    }

    pub fn realized_slippage(&self, ctx: &SlippageContext) -> Decimal {
        // In a deterministic simulation, realized slippage is based on a deterministic model.
        // We will just use expected slippage for now.
        self.expected_slippage(ctx)
    }
}
