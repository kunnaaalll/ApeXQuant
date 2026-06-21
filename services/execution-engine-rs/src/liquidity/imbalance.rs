use rust_decimal::Decimal;

pub struct OrderBookImbalance;

impl OrderBookImbalance {
    pub fn calculate(bid_volume: Decimal, ask_volume: Decimal) -> Decimal {
        let total = bid_volume + ask_volume;
        if total <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        // Returns value between -1 and 1
        // > 0 means more bids (buying pressure)
        // < 0 means more asks (selling pressure)
        ((bid_volume - ask_volume) / total).trunc_with_scale(4)
    }
}
