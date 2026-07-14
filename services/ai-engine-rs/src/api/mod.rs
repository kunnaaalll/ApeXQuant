use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Deserialize)]
pub struct InferenceRequest {
    pub symbol: String,
    pub prices: Vec<rust_decimal::Decimal>,
}

#[derive(Serialize)]
pub struct InferenceResponse {
    pub regime: String,
    pub predicted_movement: rust_decimal::Decimal,
}

pub async fn run_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/infer", post(handle_inference));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_inference(Json(payload): Json<InferenceRequest>) -> Json<InferenceResponse> {
    // Execute live deterministic logic here
    let movement = payload.prices.last().copied().unwrap_or_default() - payload.prices.first().copied().unwrap_or_default();
    
    Json(InferenceResponse {
        regime: if movement > rust_decimal::Decimal::ZERO { "TrendingUp".to_string() } else { "TrendingDown".to_string() },
        predicted_movement: movement,
    })
}
