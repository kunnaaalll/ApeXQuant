use rust_decimal::Decimal;

pub struct SanityGuard {
    pub max_leverage_limit: Decimal,
}

impl SanityGuard {
    pub fn new(max_leverage_limit: Decimal) -> Self {
        Self { max_leverage_limit }
    }

    pub fn check_leverage(
        &self,
        total_exposure: Decimal,
        account_equity: Decimal,
    ) -> Result<(), &'static str> {
        if account_equity <= Decimal::ZERO {
            return Err("Account equity is zero or negative");
        }

        let leverage = total_exposure / account_equity;
        if leverage > self.max_leverage_limit {
            return Err("Sanity check failed: leverage limit exceeded");
        }

        Ok(())
    }
}
