use crate::database::{LearningRepository, RecordLessonParams};
use apex_protos::common::{Empty, Error as CommonError, Result as CommonResult};
use apex_protos::learning::{
    learning_engine_server::LearningEngine, Lesson, LessonCollection, LessonQuery,
    ModelPerformance, ModelWeights, PerformanceQuery, RecordLessonResponse, TrainingQuery,
    TrainingRequest, TrainingRun, TrainingStatus, WeightUpdateRequest, WeightUpdateResponse,
    WeightsQuery,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct GrpcLearningEngine {
    repo: Arc<LearningRepository>,
}

impl GrpcLearningEngine {
    pub fn new(repo: Arc<LearningRepository>) -> Self {
        Self { repo }
    }
}

#[tonic::async_trait]
impl LearningEngine for GrpcLearningEngine {
    async fn record_lesson(
        &self,
        request: Request<Lesson>,
    ) -> Result<Response<RecordLessonResponse>, Status> {
        let lesson = request.into_inner();
        let lesson_id = Uuid::new_v4();

        // Convert the protobuf values to Decimals safely
        let pnl = lesson
            .outcome
            .as_ref()
            .and_then(|o| o.net_pnl.as_ref())
            .and_then(|m| m.amount.parse::<Decimal>().ok())
            .unwrap_or(Decimal::new(0, 0));
        let gross = lesson
            .outcome
            .as_ref()
            .and_then(|o| o.gross_pnl.as_ref())
            .and_then(|m| m.amount.parse::<Decimal>().ok())
            .unwrap_or(Decimal::new(0, 0));
        let entry_eff = lesson
            .analysis
            .as_ref()
            .and_then(|a| a.entry_efficiency.as_ref())
            .and_then(|m| m.value.parse::<Decimal>().ok())
            .unwrap_or(Decimal::ONE);
        let exit_eff = lesson
            .analysis
            .as_ref()
            .and_then(|a| a.exit_efficiency.as_ref())
            .and_then(|m| m.value.parse::<Decimal>().ok())
            .unwrap_or(Decimal::ONE);

        let symbol_str = lesson
            .symbol
            .as_ref()
            .map(|s| s.code.clone())
            .unwrap_or_default();

        match self
            .repo
            .record_lesson(RecordLessonParams {
                lesson_id,
                position_id: &lesson.position_id,
                signal_id: &lesson.signal_id,
                strategy_id: "system_strategy", // Default
                lesson_type: lesson.r#type().as_str_name(),
                category: lesson.category().as_str_name(),
                severity: lesson.severity,
                symbol: &symbol_str,
                market_regime: &lesson.market_regime,
                gross_pnl: gross,
                net_pnl: pnl,
                entry_efficiency: entry_eff,
                exit_efficiency: exit_eff,
            })
            .await
        {
            Ok(_) => Ok(Response::new(RecordLessonResponse {
                success: true,
                lesson_id: lesson_id.to_string(),
                predicted_value: 0.0,
                error: None,
            })),
            Err(e) => {
                tracing::error!("Failed to record lesson: {}", e);
                Ok(Response::new(RecordLessonResponse {
                    success: false,
                    lesson_id: "".to_string(),
                    predicted_value: 0.0,
                    error: Some(CommonError {
                        code: "500".to_string(),
                        message: format!("DB Error: {}", e),
                        severity: apex_protos::common::ErrorSeverity::Error.into(),
                        details: std::collections::HashMap::new(),
                        causes: vec![],
                    }),
                }))
            }
        }
    }

    async fn get_lessons(
        &self,
        _request: Request<LessonQuery>,
    ) -> Result<Response<LessonCollection>, Status> {
        // Implement retrieval
        Ok(Response::new(LessonCollection::default()))
    }

    async fn update_weights(
        &self,
        _request: Request<WeightUpdateRequest>,
    ) -> Result<Response<WeightUpdateResponse>, Status> {
        // Implement deterministic updates
        Ok(Response::new(WeightUpdateResponse::default()))
    }

    async fn get_weights(
        &self,
        _request: Request<WeightsQuery>,
    ) -> Result<Response<ModelWeights>, Status> {
        // Implement weight retrieval
        Ok(Response::new(ModelWeights::default()))
    }

    async fn start_training(
        &self,
        _request: Request<TrainingRequest>,
    ) -> Result<Response<TrainingRun>, Status> {
        // Start training pipeline
        Ok(Response::new(TrainingRun::default()))
    }

    async fn get_training_status(
        &self,
        _request: Request<TrainingQuery>,
    ) -> Result<Response<TrainingStatus>, Status> {
        // Get status
        Ok(Response::new(TrainingStatus::default()))
    }

    async fn get_model_performance(
        &self,
        _request: Request<PerformanceQuery>,
    ) -> Result<Response<ModelPerformance>, Status> {
        // Get model performance
        Ok(Response::new(ModelPerformance::default()))
    }

    async fn health(&self, _request: Request<Empty>) -> Result<Response<CommonResult>, Status> {
        Ok(Response::new(CommonResult {
            ok: true,
            error: None,
        }))
    }
}
