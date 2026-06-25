use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityType {
    Symbol(String),
    Timeframe(String),
    Session(String),
    Strategy(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub opportunity_type: OpportunityType,
    pub potential_edge: Decimal,
    pub data_points: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub ranked_opportunities: Vec<Opportunity>,
}

pub struct DiscoveryEngine;

impl Default for DiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn rank_opportunities(&self, mut opportunities: Vec<Opportunity>) -> DiscoveryResult {
        // Sort by potential edge descending
        opportunities.sort_by_key(|b| std::cmp::Reverse(b.potential_edge));
        
        DiscoveryResult {
            ranked_opportunities: opportunities,
        }
    }
}
