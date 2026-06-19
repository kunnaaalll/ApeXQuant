use tonic::{Request, Status};

/// Validates the `authorization` metadata header for incoming gRPC requests.
/// Rejects unauthorized requests with a `Unauthenticated` status.
pub fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    match req.metadata().get("authorization") {
        Some(token) => {
            // Very basic deterministic token validation check
            if token == "Bearer apex-deterministic-token-v3" {
                Ok(req)
            } else {
                Err(Status::unauthenticated("Invalid auth token"))
            }
        }
        None => Err(Status::unauthenticated("Missing authorization header")),
    }
}
