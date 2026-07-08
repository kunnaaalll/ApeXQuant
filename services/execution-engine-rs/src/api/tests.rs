use super::auth::auth_interceptor;
use super::errors::ApiError;
use super::health::health_handler;
use super::mapping::{format_decimal, parse_decimal};
use super::readiness::{readiness_handler, set_ready};
use axum::{body::Body, http::Request};
use rust_decimal::Decimal;
use std::str::FromStr;
use tonic::{metadata::MetadataValue, Request as TonicRequest, Status};
use tower::ServiceExt;

#[tokio::test]
async fn test_auth_interceptor_valid() {
    let mut req = TonicRequest::new(());
    req.metadata_mut()
        .insert("x-api-key", MetadataValue::from_static("valid-token"));
    let res = auth_interceptor(req);
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_auth_interceptor_invalid() {
    let mut req = TonicRequest::new(());
    req.metadata_mut()
        .insert("x-api-key", MetadataValue::from_static("invalid-token"));
    let res = auth_interceptor(req);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_health_endpoint() {
    let app_state = super::server::AppState {
        pg_store: None,
        redis_client: None,
        mt5_adapter: std::sync::Arc::new(crate::brokers::mt5::adapter::Mt5Adapter::new(
            "MT5".to_string(),
            "http://localhost:8001".to_string(),
        )),
        binance_adapter: std::sync::Arc::new(
            crate::brokers::binance::adapter::BinanceAdapter::new(
                "Binance".to_string(),
                "http://localhost:8002".to_string(),
                "".to_string(),
                "".to_string(),
            ),
        ),
    };
    let app = axum::Router::new()
        .route("/health", axum::routing::get(health_handler))
        .with_state(app_state);
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn test_ready_endpoint() {
    set_ready(true);
    let app_state = super::server::AppState {
        pg_store: None,
        redis_client: None,
        mt5_adapter: std::sync::Arc::new(crate::brokers::mt5::adapter::Mt5Adapter::new(
            "MT5".to_string(),
            "http://localhost:8001".to_string(),
        )),
        binance_adapter: std::sync::Arc::new(
            crate::brokers::binance::adapter::BinanceAdapter::new(
                "Binance".to_string(),
                "http://localhost:8002".to_string(),
                "".to_string(),
                "".to_string(),
            ),
        ),
    };
    let app = axum::Router::new()
        .route("/ready", axum::routing::get(readiness_handler))
        .with_state(app_state);
    let req = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), 200);
}

#[test]
fn test_error_mapping() {
    let domain_err = ApiError::NotFound("Order not found".to_string());
    let status: Status = domain_err.into();
    assert_eq!(status.code(), tonic::Code::NotFound);
    assert_eq!(status.message(), "Order not found");
}

#[test]
fn test_decimal_serialization() {
    let d = Decimal::from_str("123.45").unwrap();
    let formatted = format_decimal(d);
    assert_eq!(formatted, "123.45");

    let parsed = parse_decimal(&formatted).unwrap();
    assert_eq!(parsed, d);
}

#[tokio::test]
async fn test_determinism_100k_iterations() {
    // Execute 100k mock parses to ensure no memory leakage or panic
    for _ in 0..100_000 {
        let d = parse_decimal("123.45").unwrap();
        assert_eq!(format_decimal(d), "123.45");
    }
}
