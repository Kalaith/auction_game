use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractorTier {
    Budget,
    Reliable,
    Premium,
}

impl ContractorTier {
    pub fn label(self) -> &'static str {
        match self {
            ContractorTier::Budget => "Budget",
            ContractorTier::Reliable => "Reliable",
            ContractorTier::Premium => "Premium",
        }
    }

    pub fn cost_multiplier(self) -> f32 {
        match self {
            ContractorTier::Budget => 0.82,
            ContractorTier::Reliable => 1.0,
            ContractorTier::Premium => 1.22,
        }
    }

    pub fn value_multiplier(self) -> f32 {
        match self {
            ContractorTier::Budget => 0.88,
            ContractorTier::Reliable => 1.0,
            ContractorTier::Premium => 1.12,
        }
    }

    pub fn week_modifier(self) -> i32 {
        match self {
            ContractorTier::Budget => 1,
            ContractorTier::Reliable => 0,
            ContractorTier::Premium => -1,
        }
    }

    pub fn risk_modifier(self) -> i32 {
        match self {
            ContractorTier::Budget => 12,
            ContractorTier::Reliable => 0,
            ContractorTier::Premium => -8,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpgradeData {
    pub id: String,
    pub name: String,
    pub cost: i64,
    pub value_boost: i64,
    pub appeal_boost: i32,
    pub sale_emotion_boost: i32,
    pub removes_defect: bool,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct CompletedUpgrade {
    pub upgrade_id: String,
    pub contractor: ContractorTier,
    pub actual_cost: i64,
    pub value_boost: i64,
    pub appeal_boost: i32,
    pub sale_emotion_boost: i32,
    pub removes_defect: bool,
    pub weeks_taken: u32,
    pub note: String,
}
