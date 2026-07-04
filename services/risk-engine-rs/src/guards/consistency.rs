use rust_decimal::Decimal;

pub struct ConsistencyGuard;

impl ConsistencyGuard {
    pub fn check_limits(
        is_buy: bool,
        price: Decimal,
        stop_loss: Option<Decimal>,
        take_profit: Option<Decimal>,
    ) -> Result<(), &'static str> {
        if let Some(sl) = stop_loss {
            if is_buy && sl >= price {
                return Err("Stop loss must be below entry price for Buy orders");
            }
            if !is_buy && sl <= price {
                return Err("Stop loss must be above entry price for Sell orders");
            }
        }

        if let Some(tp) = take_profit {
            if is_buy && tp <= price {
                return Err("Take profit must be above entry price for Buy orders");
            }
            if !is_buy && tp >= price {
                return Err("Take profit must be below entry price for Sell orders");
            }
        }

        Ok(())
    }
}
