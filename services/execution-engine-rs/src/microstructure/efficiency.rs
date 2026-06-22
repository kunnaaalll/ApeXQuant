#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketEfficiency {
    Efficient,
    Normal,
    Noisy,
    Dislocated,
}

impl MarketEfficiency {
    pub fn evaluate(noise_ratio: u8) -> Result<Self, &'static str> {
        if noise_ratio > 100 {
            return Err("Noise ratio must be between 0 and 100");
        }

        if noise_ratio <= 10 {
            Ok(MarketEfficiency::Efficient)
        } else if noise_ratio <= 30 {
            Ok(MarketEfficiency::Normal)
        } else if noise_ratio <= 70 {
            Ok(MarketEfficiency::Noisy)
        } else {
            Ok(MarketEfficiency::Dislocated)
        }
    }
}
