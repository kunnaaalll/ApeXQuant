use super::events::{HealthEvent, StrategyEventWrapper};
use super::serializer::Serializer;

#[test]
fn test_serializer_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let event = StrategyEventWrapper::Health(HealthEvent {
        details: "Roundtrip test".to_string(),
    });

    let serialized = Serializer::serialize(&event)?;
    let deserialized: StrategyEventWrapper = Serializer::deserialize(serialized)?;

    assert_eq!(event, deserialized);
    Ok(())
}

#[test]
fn test_deterministic_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let event = StrategyEventWrapper::Health(HealthEvent {
        details: "Determinism check".to_string(),
    });

    let serialized1 = Serializer::serialize(&event)?;
    let serialized2 = Serializer::serialize(&event)?;

    // serde_json features `preserve_order` and `float_roundtrip` help with this.
    // Ensure that string representation is perfectly identical.
    let str1 = serde_json::to_string(&serialized1)?;
    let str2 = serde_json::to_string(&serialized2)?;

    assert_eq!(str1, str2, "Serialization is not deterministic");
    Ok(())
}

#[test]
fn test_determinism_100k_iterations() -> Result<(), Box<dyn std::error::Error>> {
    let event = StrategyEventWrapper::Health(HealthEvent {
        details: "Stress test".to_string(),
    });

    let reference = serde_json::to_string(&Serializer::serialize(&event)?)?;

    for _ in 0..100_000 {
        let serialized = Serializer::serialize(&event)?;
        let str_rep = serde_json::to_string(&serialized)?;
        assert_eq!(reference, str_rep, "Determinism failed during 100k iterations");
    }
    Ok(())
}
