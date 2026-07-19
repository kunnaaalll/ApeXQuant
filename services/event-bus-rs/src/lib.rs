#![cfg_attr(not(test), deny(unsafe_code))]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

pub mod ack;
pub mod config;
pub mod dead_letter;
pub mod dispatcher;
pub mod grpc;
pub mod health;
pub mod interceptors;
pub mod kafka;
pub mod metrics;
pub mod nats;
pub mod protobuf;
pub mod publisher;
pub mod redis;
pub mod retry;
pub mod router;
pub mod security;
pub mod serialization;
pub mod storage;
pub mod subscriber;

// Legacy/temp modules
pub mod event;
pub mod replay;
pub mod server;
pub mod shadow;
pub mod subscriptions;
pub mod topics;
pub mod validation;
