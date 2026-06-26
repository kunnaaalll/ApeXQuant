#[derive(Debug, Clone)]
pub struct CapabilityResolver {
    pub active: bool,
}

impl CapabilityResolver {
    pub fn new() -> Self {
        Self { active: true }
    }

    pub fn resolve_capability(&self, capability: &str) -> bool {
        !capability.is_empty()
    }
}

impl Default for CapabilityResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct VersionCompatibilityValidator {
    pub strict_mode: bool,
}

impl VersionCompatibilityValidator {
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    pub fn validate(&self, version: &str) -> bool {
        !version.is_empty()
    }
}

impl Default for VersionCompatibilityValidator {
    fn default() -> Self {
        Self::new(true)
    }
}
