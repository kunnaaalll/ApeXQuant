use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ClusterMetrics {
    pub name: String,
    pub quality_score: Decimal,
    pub stability: Decimal,
    pub expectancy: Decimal,
    pub members: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClusterEngine {
    pub min_members: usize,
    pub clusters: HashMap<String, ClusterMetrics>,
}

impl ClusterEngine {
    pub fn new(min_members: usize) -> Self {
        Self {
            min_members,
            clusters: HashMap::new(),
        }
    }

    pub fn evaluate_cluster(&mut self, name: String, members: Vec<String>, expectancy: Decimal, stability: Decimal) {
        if members.len() < self.min_members {
            return; // Not a valid cluster
        }

        let quality_score = expectancy * stability;
        
        self.clusters.insert(name.clone(), ClusterMetrics {
            name,
            quality_score,
            stability,
            expectancy,
            members,
        });
    }

    pub fn get_cluster(&self, name: &str) -> Option<&ClusterMetrics> {
        self.clusters.get(name)
    }
}
