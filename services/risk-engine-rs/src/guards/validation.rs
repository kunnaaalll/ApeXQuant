use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct ValidationGuard;

impl ValidationGuard {
    pub fn validate_order(volume: Decimal, price: Option<Decimal>) -> Result<(), &'static str> {
        if volume <= Decimal::ZERO {
            return Err("Order volume must be greater than zero");
        }

        if let Some(p) = price {
            if p <= Decimal::ZERO {
                return Err("Order price must be greater than zero");
            }
        }

        Ok(())
    }
}
