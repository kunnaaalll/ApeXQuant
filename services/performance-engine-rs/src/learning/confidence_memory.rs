use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct ConfidenceMemory {
    pub historical_confidence: Decimal,
    pub memory_decay: Decimal,
}

impl ConfidenceMemory {
    pub fn new(initial_confidence: Decimal, memory_decay: Decimal) -> Self {
        Self {
            historical_confidence: initial_confidence,
            memory_decay,
        }
    }

    pub fn update(&mut self, recent_confidence: Decimal) -> Decimal {
        self.historical_confidence = (self.historical_confidence * self.memory_decay) + 
                                     (recent_confidence * (dec!(1.0) - self.memory_decay));
        self.historical_confidence
    }
}
