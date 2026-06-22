use tower::ServiceBuilder;
use tower::layer::util::{Identity, Stack};

use super::logging::LoggingLayer;
use super::tracing::TracingLayer;

pub type MiddlewareStack = Stack<LoggingLayer, Stack<TracingLayer, Identity>>;

pub fn build_middleware_stack() -> MiddlewareStack {
    ServiceBuilder::new()
        .layer(TracingLayer)
        .layer(LoggingLayer)
        .into_inner()
}
