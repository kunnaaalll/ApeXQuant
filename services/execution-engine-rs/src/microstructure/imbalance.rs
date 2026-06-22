use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImbalanceScore {
    pub bid_volume: Decimal,
    pub ask_volume: Decimal,
    pub score: u8, // 0-100
}

impl ImbalanceScore {
    pub fn calculate(bid_volume: Decimal, ask_volume: Decimal) -> Result<Self, &'static str> {
        if bid_volume < Decimal::ZERO || ask_volume < Decimal::ZERO {
            return Err("Volumes cannot be negative");
        }

        let total = bid_volume + ask_volume;
        let score = if total == Decimal::ZERO {
            50 // Neutral if no volume
        } else {
            // (bid / total) * 100 -> rounded
            let ratio = (bid_volume / total) * Decimal::new(100, 0);
            
            // convert decimal to u8 safely
            let rounded = ratio.round();
            use rust_decimal::prelude::ToPrimitive;
            if let Some(val) = rounded.to_u64() {
                val.clamp(0, 100) as u8
            } else {
                50 // fallback
            }
        };

        Ok(Self {
            bid_volume,
            ask_volume,
            score,
        })
    }
}
