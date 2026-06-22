use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{info, span, Level};
use std::time::Instant;
use tonic::codegen::http;

#[derive(Clone)]
pub struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    inner: S,
}

impl<S, ReqBody> Service<http::Request<ReqBody>> for LoggingMiddleware<S>
where
    S: Service<http::Request<ReqBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let path = req.uri().path().to_string();
        let method = req.method().to_string();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let start = Instant::now();
            let span = span!(Level::INFO, "request", method = %method, path = %path);
            let _enter = span.enter();
            
            info!("Incoming request");

            let result = inner.call(req).await;
            
            let duration = start.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                "Request completed"
            );

            result
        })
    }
}
