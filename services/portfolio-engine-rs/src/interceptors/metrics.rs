use tonic::{Request, Status};

pub fn metrics_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    // In a real implementation, we would increment prometheus counters here
    // based on the request path or metadata.
    Ok(request)
}
