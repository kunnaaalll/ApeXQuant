#![cfg_attr(not(test), deny(unsafe_code))]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

pub mod config;
pub mod dispatcher;
pub mod grpc;
pub mod metrics;
pub mod protobuf;
pub mod redis;
pub mod retry;
pub mod router;
pub mod security;
pub mod serialization;
pub mod storage;
pub mod subscriber;

// Legacy/temp modules
pub mod server;
pub mod shadow;
pub mod topics;
pub mod validation;
