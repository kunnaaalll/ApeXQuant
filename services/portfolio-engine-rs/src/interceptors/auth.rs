use tonic::{Request, Status};

pub fn auth_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    match request.metadata().get("authorization") {
        Some(_t) => {
            // Validate token here
            Ok(request)
        }
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}
