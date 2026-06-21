use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::execution::ExecutionRequest;
use crate::order::{OrderSide, OrderType};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    #[error("Size must be greater than 0")]
    InvalidSize,
    #[error("Price must be greater than 0")]
    InvalidPrice,
    #[error("Stop loss must be lower than price for Buy orders and higher for Sell orders")]
    InvalidStopLoss,
    #[error("Take profit must be higher than price for Buy orders and lower for Sell orders")]
    InvalidTakeProfit,
    #[error("Negative price is not allowed")]
    NegativePrice,
}

pub struct OrderValidator;

impl OrderValidator {
    pub fn validate_request(
        req: &ExecutionRequest,
        stop_loss: Option<Decimal>,
        take_profit: Option<Decimal>,
    ) -> Result<(), ValidationError> {
        let zero = dec!(0);

        if req.size <= zero {
            return Err(ValidationError::InvalidSize);
        }

        if let Some(price) = req.price {
            if price < zero {
                return Err(ValidationError::NegativePrice);
            }
            if price == zero {
                return Err(ValidationError::InvalidPrice);
            }

            if req.side == OrderSide::Buy {
                if let Some(sl) = stop_loss {
                    if sl >= price {
                        return Err(ValidationError::InvalidStopLoss);
                    }
                    if sl < zero {
                        return Err(ValidationError::NegativePrice);
                    }
                }
                if let Some(tp) = take_profit {
                    if tp <= price {
                        return Err(ValidationError::InvalidTakeProfit);
                    }
                }
            } else if req.side == OrderSide::Sell {
                if let Some(sl) = stop_loss {
                    if sl <= price {
                        return Err(ValidationError::InvalidStopLoss);
                    }
                }
                if let Some(tp) = take_profit {
                    if tp >= price {
                        return Err(ValidationError::InvalidTakeProfit);
                    }
                    if tp < zero {
                        return Err(ValidationError::NegativePrice);
                    }
                }
            }
        } else if req.order_type == OrderType::Limit
            || req.order_type == OrderType::Stop
            || req.order_type == OrderType::StopLimit
        {
            return Err(ValidationError::InvalidPrice);
        }

        Ok(())
    }
}
