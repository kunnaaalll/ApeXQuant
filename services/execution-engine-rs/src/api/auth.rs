use tonic::{Request, Status};

pub fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    match req.metadata().get("x-api-key") {
        Some(t) if t == "valid-token" => Ok(req),
        _ => Err(Status::permission_denied("Invalid or missing API key")),
    }
}
