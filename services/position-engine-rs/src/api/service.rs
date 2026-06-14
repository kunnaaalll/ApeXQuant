// Assuming protobuf definitions are in `apex-protos`
// use apex_protos::position::v1::position_engine_server::PositionEngine;

pub struct PositionEngineService {
    // registry: std::sync::Arc<crate::positions::PositionRegistry>,
    // store: std::sync::Arc<crate::storage::PostgresStore>,
}

impl PositionEngineService {
    pub fn new() -> Self {
        Self {}
    }
}

// TODO: Implement the actual gRPC service trait once protos are linked.
// #[tonic::async_trait]
// impl PositionEngine for PositionEngineService {
//     async fn open_position(...) -> Result<tonic::Response<...>, tonic::Status> { ... }
//     async fn update_position(...) -> Result<tonic::Response<...>, tonic::Status> { ... }
//     async fn close_position(...) -> Result<tonic::Response<...>, tonic::Status> { ... }
//     async fn get_position(...) -> Result<tonic::Response<...>, tonic::Status> { ... }
// }
