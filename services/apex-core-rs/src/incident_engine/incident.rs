#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentLevel {
    Info,
    Warning,
    Critical,
    Catastrophic,
}

#[derive(Debug, Clone)]
pub struct Incident {
    pub level: IncidentLevel,
    pub description: String,
    pub service_id: String,
}

#[derive(Debug, Clone)]
pub struct IncidentEngine {
    pub active_incidents: Vec<Incident>,
}

impl IncidentEngine {
    pub fn new() -> Self {
        Self {
            active_incidents: Vec::new(),
        }
    }

    pub fn report_incident(&mut self, incident: Incident) {
        self.process_incident(&incident);
        self.active_incidents.push(incident);
    }

    fn process_incident(&self, incident: &Incident) {
        match incident.level {
            IncidentLevel::Info => self.notify_operators(),
            IncidentLevel::Warning => self.notify_operators(),
            IncidentLevel::Critical => {
                self.notify_operators();
                self.trigger_recovery(&incident.service_id);
            }
            IncidentLevel::Catastrophic => {
                self.notify_operators();
                self.freeze_trading();
                self.isolate_services(&incident.service_id);
                self.trigger_recovery(&incident.service_id);
            }
        }
        self.generate_incident_report(incident);
    }

    fn notify_operators(&self) {}
    fn freeze_trading(&self) {}
    fn isolate_services(&self, _service_id: &str) {}
    fn trigger_recovery(&self, _service_id: &str) {}
    fn generate_incident_report(&self, _incident: &Incident) {}
}

impl Default for IncidentEngine {
    fn default() -> Self {
        Self::new()
    }
}
