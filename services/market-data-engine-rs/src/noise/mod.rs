use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoiseState {
    Clean,
    Moderate,
    Noisy,
    ExtremeNoise,
}

pub struct NoiseMetrics {
    pub ratio: Decimal,
    pub state: NoiseState,
}

pub struct NoiseEngine;

impl NoiseEngine {
    pub fn calculate(net_displacement: Decimal, total_path: Decimal) -> Result<NoiseMetrics, &'static str> {
        if net_displacement < Decimal::ZERO || total_path < Decimal::ZERO {
            return Err("Distances cannot be negative");
        }
        
        let ratio = if total_path.is_zero() {
            Decimal::ZERO
        } else {
            Decimal::ONE - (net_displacement / total_path)
        };

        let state = match ratio {
            r if r > Decimal::new(80, 2) => NoiseState::ExtremeNoise,
            r if r > Decimal::new(50, 2) => NoiseState::Noisy,
            r if r > Decimal::new(20, 2) => NoiseState::Moderate,
            _ => NoiseState::Clean,
        };

        Ok(NoiseMetrics {
            ratio,
            state,
        })
    }
}
