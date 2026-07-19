use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RequestRouter {
    pub routing_table: HashMap<String, String>,
}

impl RequestRouter {
    pub fn new() -> Self {
        Self {
            routing_table: HashMap::new(),
        }
    }

    pub fn route_request(&self, request_id: &str) -> Option<String> {
        self.routing_table.get(request_id).cloned()
    }

    pub fn add_route(&mut self, source: String, target: String) {
        self.routing_table.insert(source, target);
    }
}

impl Default for RequestRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LoadBalancer {
    pub endpoints: Vec<String>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            endpoints: Vec::new(),
        }
    }

    pub fn next_endpoint(&self) -> Option<String> {
        self.endpoints.first().cloned()
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ServiceAffinityEngine {
    pub active: bool,
}

impl ServiceAffinityEngine {
    pub fn new() -> Self {
        Self { active: true }
    }
}

impl Default for ServiceAffinityEngine {
    fn default() -> Self {
        Self::new()
    }
}
