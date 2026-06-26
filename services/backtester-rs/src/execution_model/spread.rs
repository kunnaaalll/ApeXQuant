use rust_decimal::Decimal;

pub struct SpreadContext {
    pub base_spread: Decimal,
    pub volatility_factor: Decimal,
    pub is_out_of_session: bool,
}

pub struct SpreadModel {
    pub session_multiplier: Decimal,
}

impl SpreadModel {
    pub fn new(session_multiplier: Decimal) -> Self {
        Self { session_multiplier }
    }

    pub fn calculate_spread(&self, ctx: &SpreadContext) -> Decimal {
        let mut spread = ctx.base_spread * ctx.volatility_factor;
        if ctx.is_out_of_session {
            spread *= self.session_multiplier;
        }
        spread
    }
}
