use tonic::{Request, Status};

#[allow(clippy::result_large_err)]
pub fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    match req.metadata().get("authorization") {
        Some(token) => {
            if token.is_empty() {
                Err(Status::unauthenticated("Invalid authorization token"))
            } else {
                Ok(req)
            }
        }
        None => Err(Status::unauthenticated("Missing authorization metadata")),
    }
}
