use std::time::Instant;
use tower::{Layer, Service};
use tracing::{info, span, Level};

#[derive(Clone)]
pub struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> <Self as Layer<S>>::Service {
        LoggingService { inner }
    }
}

#[derive(Clone)]
pub struct LoggingService<S> {
    inner: S,
}

impl<S, ReqBody> Service<axum::http::Request<ReqBody>> for LoggingService<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = axum::http::Response<tonic::body::BoxBody>>
        + Clone
        + Send
        + 'static,
    <S as Service<axum::http::Request<ReqBody>>>::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = <S as Service<axum::http::Request<ReqBody>>>::Response;
    type Error = <S as Service<axum::http::Request<ReqBody>>>::Error;
    type Future = futures::future::BoxFuture<
        'static,
        Result<
            <Self as Service<axum::http::Request<ReqBody>>>::Response,
            <Self as Service<axum::http::Request<ReqBody>>>::Error,
        >,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), <Self as Service<axum::http::Request<ReqBody>>>::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(
        &mut self,
        req: axum::http::Request<ReqBody>,
    ) -> <Self as Service<axum::http::Request<ReqBody>>>::Future {
        let method = req.method().to_string();
        let uri = req.uri().to_string();

        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let start = Instant::now();

            let span = span!(Level::INFO, "grpc_request", method = %method, uri = %uri);
            let _enter = span.enter();

            let res = inner.call(req).await;

            let duration = start.elapsed();
            info!(duration_ms = duration.as_millis(), "Request completed");

            res
        })
    }
}
