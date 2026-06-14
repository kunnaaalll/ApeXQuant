use std::sync::atomic::{AtomicBool, Ordering};

pub struct HealthCheck {
    is_healthy: AtomicBool,
}

impl HealthCheck {
    pub fn new() -> Self {
        Self {
            is_healthy: AtomicBool::new(true),
        }
    }

    pub fn set_unhealthy(&self) {
        self.is_healthy.store(false, Ordering::Relaxed);
    }

    pub fn is_healthy(&self) -> bool {
        self.is_healthy.load(Ordering::Relaxed)
    }
}
