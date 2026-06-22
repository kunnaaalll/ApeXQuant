#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketQuality {
    pub liquidity_score: u8, // 0-100
    pub volatility_score: u8, // 0-100
    pub overall_quality: u8, // 0-100
}

impl MarketQuality {
    pub fn calculate(liquidity_score: u8, volatility_score: u8) -> Result<Self, &'static str> {
        if liquidity_score > 100 || volatility_score > 100 {
            return Err("Scores must be bounded between 0 and 100");
        }

        let overall = ((liquidity_score as u16 + (100 - volatility_score as u16)) / 2) as u8;

        Ok(Self {
            liquidity_score,
            volatility_score,
            overall_quality: overall,
        })
    }
}
