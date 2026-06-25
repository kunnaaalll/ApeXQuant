use apex_protos::events::Event;

pub enum DeliveryGuarantee {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl Default for DeliveryGuarantee {
    fn default() -> Self {
        Self::AtLeastOnce
    }
}

pub struct DeliveryManager;

impl DeliveryManager {
    pub fn determine_guarantee(event: &Event) -> DeliveryGuarantee {
        // Enforce ExactlyOnce for critical domains
        if event.topic.starts_with("execution.") || 
           event.topic == "risk.freeze" || 
           event.topic == "portfolio.allocation" {
            return DeliveryGuarantee::ExactlyOnce;
        }

        // Enforce AtMostOnce for high-throughput, loss-tolerant streams
        if event.topic.starts_with("market.tick") {
            return DeliveryGuarantee::AtMostOnce;
        }

        DeliveryGuarantee::AtLeastOnce
    }

    pub fn required_durability(guarantee: &DeliveryGuarantee) -> apex_protos::events::DurabilityLevel {
        match guarantee {
            DeliveryGuarantee::AtMostOnce => apex_protos::events::DurabilityLevel::DurabilityMemoryOnly,
            DeliveryGuarantee::AtLeastOnce => apex_protos::events::DurabilityLevel::DurabilityDisk,
            DeliveryGuarantee::ExactlyOnce => apex_protos::events::DurabilityLevel::DurabilitySyncReplica,
        }
    }
}
