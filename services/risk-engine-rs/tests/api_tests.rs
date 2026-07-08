use risk_engine::api::error::ApiError;
use tonic::{Request, Status, Code};
use tokio_stream::StreamExt;

use apex_protos::risk::risk_engine_server::RiskEngine;
use apex_protos::risk::*;
use risk_engine::api::risk_service::RiskServiceImpl;
use risk_engine::interceptors::auth::auth_interceptor;

#[test]
fn test_error_mapping() {
    let err_not_found = ApiError::NotFound;
    let status: Status = err_not_found.into();
    assert_eq!(status.code(), Code::NotFound);

    let err_invalid = ApiError::InvalidArgument;
    let status: Status = err_invalid.into();
    assert_eq!(status.code(), Code::InvalidArgument);

    let err_unauth = ApiError::Unauthorized;
    let status: Status = err_unauth.into();
    assert_eq!(status.code(), Code::Unauthenticated);

    let err_internal = ApiError::Internal;
    let status: Status = err_internal.into();
    assert_eq!(status.code(), Code::Internal);
}

#[tokio::test]
async fn test_streaming() {
    let service = RiskServiceImpl::new(risk_engine::api::risk_service::RiskState::new(), None, None);
    let request = Request::new(EventQuery {
        account_id: "acc_123".to_string(),
        start_time: None,
        end_time: None,
    });

    let response = service.load_events(request).await.unwrap();
    let mut stream = response.into_inner();
    
    // Stream should be valid (empty because it's a mock)
    let next = stream.next().await;
    assert!(next.is_none());
}

#[tokio::test]
async fn test_determinism() {
    let service = RiskServiceImpl::new(risk_engine::api::risk_service::RiskState::new(), None, None);
    
    // Simulate 100,000 requests
    for i in 0..100_000 {
        let request = Request::new(RiskStateQuery {
            account_id: format!("acc_{}", i % 10),
        });
        
        let response = service.get_risk_state(request).await.unwrap().into_inner();
        
        // Assert determinism
        assert_eq!(response.state, "Normal");
        assert_eq!(response.account_id, format!("acc_{}", i % 10));
    }
}

#[test]
fn test_no_panics_auth_interceptor() {
    // Missing metadata
    let request = Request::new(());
    let res = auth_interceptor(request);
    assert_eq!(res.unwrap_err().code(), Code::Unauthenticated);
    
    // Valid metadata
    let mut request = Request::new(());
    request.metadata_mut().insert("authorization", "Bearer token".parse().unwrap());
    let res = auth_interceptor(request);
    assert!(res.is_ok());

    // Empty metadata string
    let mut request = Request::new(());
    request.metadata_mut().insert("authorization", "".parse().unwrap());
    let res = auth_interceptor(request);
    assert_eq!(res.unwrap_err().code(), Code::Unauthenticated);
}
