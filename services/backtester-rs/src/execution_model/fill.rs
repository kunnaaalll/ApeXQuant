use rust_decimal::Decimal;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FillStatus {
    Full,
    Partial,
    Delayed,
    Rejected(String),
}

#[derive(Debug, Clone)]
pub struct FillRequest {
    pub order_id: String,
    pub requested_size: Decimal,
    pub limit_price: Option<Decimal>,
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct FillResult {
    pub order_id: String,
    pub status: FillStatus,
    pub filled_size: Decimal,
    pub fill_price: Decimal,
    pub timestamp: OffsetDateTime,
}

pub struct FillEngine {
    pub max_fill_size: Decimal,
}

impl FillEngine {
    pub fn new(max_fill_size: Decimal) -> Self {
        Self { max_fill_size }
    }

    pub fn process_fill(
        &self,
        req: &FillRequest,
        current_price: Decimal,
        available_liquidity: Decimal,
    ) -> FillResult {
        if available_liquidity <= Decimal::ZERO {
            return FillResult {
                order_id: req.order_id.clone(),
                status: FillStatus::Rejected("No liquidity".to_string()),
                filled_size: Decimal::ZERO,
                fill_price: Decimal::ZERO,
                timestamp: req.timestamp,
            };
        }

        let mut fill_size = req.requested_size;
        let mut status = FillStatus::Full;

        if fill_size > available_liquidity {
            fill_size = available_liquidity;
            status = FillStatus::Partial;
        }

        if fill_size > self.max_fill_size {
            fill_size = self.max_fill_size;
            status = FillStatus::Partial;
        }

        FillResult {
            order_id: req.order_id.clone(),
            status,
            filled_size: fill_size,
            fill_price: req.limit_price.unwrap_or(current_price),
            timestamp: req.timestamp,
        }
    }
}
