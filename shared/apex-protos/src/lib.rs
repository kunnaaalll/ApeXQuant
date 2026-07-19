//! APEX V3 Protocol Buffers
//!
//! Generated gRPC clients and servers for APEX services.

#![allow(missing_docs)]
#![allow(clippy::large_enum_variant)]

// Protocol buffer modules will be generated here
pub mod common {
    include!("generated/apex.common.rs");
}
pub mod signal {
    include!("generated/apex.signal.rs");
}
pub mod execution {
    include!("generated/apex.execution.rs");
}
pub mod risk {
    include!("generated/apex.risk.rs");
}
pub mod portfolio {
    include!("generated/apex.portfolio.rs");
}
pub mod analytics {
    include!("generated/apex.analytics.rs");
}
pub mod events {
    include!("generated/apex.events.rs");
}
pub mod learning {
    include!("generated/apex.learning.rs");
}
pub mod strategy {
    include!("generated/apex.strategy.rs");
}
pub mod position {
    include!("generated/apex.position.rs");
}

/// Placeholder until protobufs are generated
pub const VERSION: &str = "3.0.0";
