use crate::spread::SpreadMetrics;
use crate::depth::DepthMetrics;
use crate::imbalance::ImbalanceMetrics;

pub struct MicrostructureMetrics {
    pub spread: SpreadMetrics,
    pub depth: DepthMetrics,
    pub imbalance: ImbalanceMetrics,
}

pub struct MicrostructureEngine;

impl MicrostructureEngine {
    pub fn compile(spread: SpreadMetrics, depth: DepthMetrics, imbalance: ImbalanceMetrics) -> Result<MicrostructureMetrics, &'static str> {
        Ok(MicrostructureMetrics {
            spread,
            depth,
            imbalance,
        })
    }
}
