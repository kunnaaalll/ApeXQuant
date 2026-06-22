use std::task::{Context, Poll};
use tower::{Layer, Service};
use uuid::Uuid;
use tonic::codegen::http;
use tonic::codegen::http::header::HeaderName;

#[derive(Clone)]
pub struct TracingLayer;

impl<S> Layer<S> for TracingLayer {
    type Service = TracingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TracingMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct TracingMiddleware<S> {
    inner: S,
}

impl<S, ReqBody> Service<http::Request<ReqBody>> for TracingMiddleware<S>
where
    S: Service<http::Request<ReqBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<ReqBody>) -> Self::Future {
        // Ensure x-request-id is present
        let request_id_header = HeaderName::from_static("x-request-id");
        if req.headers().get(&request_id_header).is_none() {
            let uuid = Uuid::new_v4().to_string();
            if let Ok(header_value) = uuid.parse() {
                req.headers_mut().insert(request_id_header, header_value);
            }
        }
        
        let correlation_id_header = HeaderName::from_static("x-correlation-id");
        if req.headers().get(&correlation_id_header).is_none() {
            let uuid = Uuid::new_v4().to_string();
            if let Ok(header_value) = uuid.parse() {
                req.headers_mut().insert(correlation_id_header, header_value);
            }
        }

        self.inner.call(req)
    }
}
