#![cfg_attr(not(test), deny(unsafe_code))]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

pub mod publisher;
pub mod subscriber;
pub mod router;
pub mod dispatcher;
pub mod serialization;
pub mod protobuf;
pub mod grpc;
pub mod nats;
pub mod redis;
pub mod kafka;
pub mod ack;
pub mod retry;
pub mod dead_letter;
pub mod health;
pub mod metrics;
pub mod storage;
pub mod config;
pub mod security;
pub mod interceptors;

// Legacy/temp modules
pub mod event;
pub mod subscriptions;
pub mod validation;
pub mod replay;
pub mod shadow;
pub mod server;
pub mod topics;
