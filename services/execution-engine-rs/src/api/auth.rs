use tonic::{Request, Status};

#[allow(clippy::result_large_err)]
pub fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    match req.metadata().get("x-api-key") {
        Some(t) if t == "valid-token" => Ok(req),
        _ => Err(Status::permission_denied("Invalid or missing API key")),
    }
}
