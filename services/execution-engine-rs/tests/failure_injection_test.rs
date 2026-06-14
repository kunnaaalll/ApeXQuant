use std::time::Duration;
use execution_engine::retry::policies::{FailureClass, RetryPolicy};

#[tokio::test]
async fn test_broker_network_drop() {
    // Mocking a network drop failure sequence
    // A retry policy that retries up to 3 times for transient failures
    let policy = RetryPolicy {
        max_retries: 3,
        requires_idempotency_key: true,
        rate_limit_backoff: Duration::from_secs(1),
    };

    // Simulate attempts
    let mut attempt = 0;
    
    // Attempt 1: Network drop (Transient)
    let err1 = FailureClass::Transient;
    assert!(policy.should_retry(attempt, &err1));
    attempt += 1;

    // Attempt 2: Network drop (Transient)
    let err2 = FailureClass::Transient;
    assert!(policy.should_retry(attempt, &err2));
    attempt += 1;

    // Attempt 3: Server recovers but returns 400 Bad Request (Permanent)
    let err3 = FailureClass::Permanent;
    assert!(!policy.should_retry(attempt, &err3)); // Should NOT retry on permanent error even if max_retries not reached

    // If it was still transient, it would retry until max_retries
    let err4 = FailureClass::Transient;
    assert!(policy.should_retry(attempt, &err4)); 
    attempt += 1;

    // Attempt 4: Max retries reached
    assert!(!policy.should_retry(attempt, &err4));
}
