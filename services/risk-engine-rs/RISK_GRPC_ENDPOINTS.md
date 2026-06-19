# Risk gRPC Endpoints

The `RiskEngine` service exposes the following deterministic endpoints via `tonic` gRPC:

## Unary
- `GetRiskState(RiskStateQuery)` -> `RiskStateResponse`
- `GetDrawdown(DrawdownQuery)` -> `DrawdownResponse`
- `GetExposure(ExposureQuery)` -> `ExposureResponse`
- `GetCorrelation(CorrelationQuery)` -> `CorrelationResponse`
- `GetHiddenLeverage(HiddenLeverageQuery)` -> `HiddenLeverageResponse`
- `GetHistoricalVar(VarQuery)` -> `VarResponse`
- `GetParametricVar(VarQuery)` -> `VarResponse`
- `GetExpectedShortfall(VarQuery)` -> `ExpectedShortfallResponse`
- `GetCircuitBreaker(CircuitBreakerQuery)` -> `CircuitBreakerResponse`
- `GetRecommendation(RecommendationQuery)` -> `RecommendationResponse`
- `GetStressAssessment(StressQuery)` -> `StressResponse`

## Streaming
- `LoadEvents(EventQuery)` -> `stream RiskEvent`
- `SubscribeEvents(EventSubscription)` -> `stream RiskEvent`

All endpoints are guarded by the AuthInterceptor and pass through Logging/Metrics layers.
