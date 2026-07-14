use anyhow::{Result, Context};
use backoff::{ExponentialBackoff, Error as BackoffError, future::retry};
use std::time::Duration;

#[derive(Clone)]
pub struct RetryEngine {
    max_elapsed_time: Duration,
}

impl RetryEngine {
    pub fn new(max_elapsed_time_secs: u64) -> Self {
        Self {
            max_elapsed_time: Duration::from_secs(max_elapsed_time_secs),
        }
    }

    pub async fn execute_with_retry<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, BackoffError<anyhow::Error>>>,
    {
        let backoff = ExponentialBackoff {
            max_elapsed_time: Some(self.max_elapsed_time),
            ..Default::default()
        };

        retry(backoff, || operation())
            .await
            .context("Operation failed after maximum retries")
    }
}
