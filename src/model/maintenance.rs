use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum MaintenanceKind {
    PlumbingLeak,
    HeatingFailure,
    RoofRepair,
}

impl MaintenanceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::PlumbingLeak => "Plumbing Leak",
            Self::HeatingFailure => "Heating Failure",
            Self::RoofRepair => "Roof Repair",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MaintenanceIssue {
    pub kind: MaintenanceKind,
    pub repair_cost: i64,
    pub weekly_rent_loss: i64,
    pub description: String,
}
