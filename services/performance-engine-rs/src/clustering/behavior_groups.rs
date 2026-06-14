use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BehaviorGroups {
    pub risk_on_cluster: Vec<String>,
    pub risk_off_cluster: Vec<String>,
    pub high_volatility_cluster: Vec<String>,
    pub trending_cluster: Vec<String>,
    pub cluster_scores: HashMap<String, Decimal>,
}

impl BehaviorGroups {
    pub fn new() -> Self {
        Self {
            risk_on_cluster: Vec::new(),
            risk_off_cluster: Vec::new(),
            high_volatility_cluster: Vec::new(),
            trending_cluster: Vec::new(),
            cluster_scores: HashMap::new(),
        }
    }

    pub fn set_score(&mut self, cluster_name: String, score: Decimal) {
        self.cluster_scores.insert(cluster_name, score);
    }
}
