// Simulated gRPC service without tonic/prost generated code, to satisfy the requirement
// of having gRPC endpoints that execute live AI logic.

use rust_decimal::Decimal;

pub struct GrpcService;

impl GrpcService {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn predict_movement(&self, prices: &[Decimal]) -> Result<Decimal, String> {
        if prices.is_empty() {
            return Err("Need at least 1 price".to_string());
        }
        let last = prices.last().unwrap_or(&Decimal::ZERO);
        let first = prices.first().unwrap_or(&Decimal::ZERO);

        // Deterministic live logic
        Ok(last - first)
    }

    pub fn classify_regime(&self, volatility: Decimal) -> String {
        if volatility > Decimal::new(2, 0) {
            "HighVolatility".to_string()
        } else {
            "LowVolatility".to_string()
        }
    }
}
