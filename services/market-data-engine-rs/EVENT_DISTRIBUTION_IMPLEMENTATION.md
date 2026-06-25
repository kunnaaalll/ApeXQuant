# Event Distribution Layer Implementation

## Overview
The Event Distribution Layer is the backbone for routing deterministic market intelligence across the APEX V3 ecosystem. It ensures that consumers such as Strategy, Risk, Execution, and Portfolio engines receive the data they need with configurable delivery guarantees.

## Components
- **MarketEventPublisher**: Implements broadcast channels for asynchronous message distribution.
- **MarketEventSubscriber**: Provides a receiver interface for downstream systems to consume specific topics.
- **Topics**: `TickEvents`, `CandleEvents`, `VolatilityEvents`, `RegimeEvents`, `CorrelationEvents`, `IntelligenceEvents`, `QualityEvents`, `SessionEvents`.
- **DeliveryGuarantee**: Supports `AtLeastOnce`, `ExactlyOnce`, `Replayable`, and `Ordered`.

## Guarantees
- Zero f32/f64, utilizing `rust_decimal::Decimal`.
- Safe code only.
- Designed for high-throughput memory-safe distribution using `tokio::sync::broadcast`.
