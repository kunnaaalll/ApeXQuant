use metrics::{counter, histogram};
use std::time::Instant;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, inner: S) -> <Self as Layer<S>>::Service {
        MetricsService { inner }
    }
}

#[derive(Clone)]
pub struct MetricsService<S> {
    inner: S,
}

impl<S, ReqBody> Service<axum::http::Request<ReqBody>> for MetricsService<S>
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
        let uri = req.uri().path().to_string();

        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let start = Instant::now();
            counter!("grpc_requests_total", "path" => uri.clone()).increment(1);

            let res = inner.call(req).await;

            let duration = start.elapsed();
            histogram!("grpc_request_duration_seconds", "path" => uri.clone())
                .record(duration.as_secs_f64());

            if res.is_err() {
                counter!("grpc_errors_total", "path" => uri.clone()).increment(1);
            } else if let Ok(response) = &res {
                if let Some(status) = response.headers().get("grpc-status") {
                    if status != "0" {
                        counter!("grpc_errors_total", "path" => uri.clone()).increment(1);
                    }
                }
            }

            res
        })
    }
}
