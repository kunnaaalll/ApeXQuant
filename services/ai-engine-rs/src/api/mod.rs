use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Deserialize)]
pub struct InferenceRequest {
    pub symbol: String,
    pub prices: Vec<rust_decimal::Decimal>,
}

#[derive(Serialize)]
pub struct InferenceResponse {
    pub error: &'static str,
}

pub async fn run_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/infer", post(handle_inference));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_inference(Json(_payload): Json<InferenceRequest>) -> (StatusCode, Json<InferenceResponse>) {
    // No production model artifact is present in this repository. Do not derive a
    // prediction from price deltas and present it as model inference.
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(InferenceResponse { error: "MODEL_UNAVAILABLE" }),
    )
}
