use tonic::{Request, Status};
use tracing::info;

#[derive(Clone)]
pub struct LoggingInterceptor;

impl tonic::service::Interceptor for LoggingInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        info!("Received request: {:?}", request.metadata());
        // For actual duration, we'd need tower middleware, but this logs incoming requests.
        Ok(request)
    }
}

pub fn logging_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    info!("gRPC call: {:?}", request.metadata());
    Ok(request)
}
