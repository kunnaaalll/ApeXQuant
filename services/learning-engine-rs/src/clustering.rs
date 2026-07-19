use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterType {
    Strategy,
    Market,
    Failure,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub id: String,
    pub cluster_type: ClusterType,
    pub members: Vec<String>,
    pub properties: HashMap<String, String>,
}

pub struct ClusterManager {
    clusters: Vec<Cluster>,
}

impl Default for ClusterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClusterManager {
    pub fn new() -> Self {
        Self { clusters: vec![] }
    }

    pub fn add_cluster(&mut self, cluster: Cluster) {
        self.clusters.push(cluster);
    }

    pub fn get_clusters_by_type(&self, cluster_type: ClusterType) -> Vec<Cluster> {
        self.clusters
            .iter()
            .filter(|c| {
                std::mem::discriminant(&c.cluster_type) == std::mem::discriminant(&cluster_type)
            })
            .cloned()
            .collect()
    }

    // A deterministic clustering mechanism would group items by similar traits without f32/f64.
    pub fn cluster_strategies(&self, traits: &HashMap<String, Vec<String>>) -> Vec<Cluster> {
        let mut result = vec![];
        // deterministic bucketing implementation
        for (trait_key, members) in traits {
            let mut props = HashMap::new();
            props.insert("trait".to_string(), trait_key.clone());
            result.push(Cluster {
                id: format!("cluster_{}", trait_key),
                cluster_type: ClusterType::Strategy,
                members: members.clone(),
                properties: props,
            });
        }
        result
    }
}
