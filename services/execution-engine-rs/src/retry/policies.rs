use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureClass {
    Transient,    // e.g. Timeout, 5xx server errors, network drop
    Permanent,    // e.g. 400 Bad Request, Validation error, Insufficient Funds
    RateLimit,    // 429 Too Many Requests
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: usize,
    pub requires_idempotency_key: bool,
    pub rate_limit_backoff: Duration, // explicit delay when encountering 429
}

impl RetryPolicy {
    pub fn should_retry(&self, attempt: usize, failure_class: &FailureClass) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        match failure_class {
            FailureClass::Transient => true,
            FailureClass::Permanent => false,
            FailureClass::RateLimit => true,
        }
    }
}
