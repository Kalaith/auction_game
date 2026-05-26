use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ResearchLevel {
    StreetScan,
    AgentPack,
    BuildingInspection,
    FullDiligence,
}

impl ResearchLevel {
    pub fn label(self) -> &'static str {
        match self {
            ResearchLevel::StreetScan => "Street Scan",
            ResearchLevel::AgentPack => "Agent Pack",
            ResearchLevel::BuildingInspection => "Building Inspection",
            ResearchLevel::FullDiligence => "Full Diligence",
        }
    }

    pub fn cost(self) -> i64 {
        match self {
            ResearchLevel::StreetScan => 0,
            ResearchLevel::AgentPack => 1_500,
            ResearchLevel::BuildingInspection => 3_500,
            ResearchLevel::FullDiligence => 6_000,
        }
    }

    pub fn confidence_label(self) -> &'static str {
        match self {
            ResearchLevel::StreetScan => "low confidence",
            ResearchLevel::AgentPack => "medium confidence",
            ResearchLevel::BuildingInspection => "high confidence",
            ResearchLevel::FullDiligence => "very high confidence",
        }
    }

    pub fn range_width(self) -> f32 {
        match self {
            ResearchLevel::StreetScan => 0.09,
            ResearchLevel::AgentPack => 0.06,
            ResearchLevel::BuildingInspection => 0.045,
            ResearchLevel::FullDiligence => 0.028,
        }
    }

    pub fn next_levels(self) -> &'static [ResearchLevel] {
        match self {
            ResearchLevel::StreetScan => &[
                ResearchLevel::AgentPack,
                ResearchLevel::BuildingInspection,
                ResearchLevel::FullDiligence,
            ],
            ResearchLevel::AgentPack => &[
                ResearchLevel::BuildingInspection,
                ResearchLevel::FullDiligence,
            ],
            ResearchLevel::BuildingInspection => &[ResearchLevel::FullDiligence],
            ResearchLevel::FullDiligence => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum WalkawayStyle {
    Conservative,
    Balanced,
    Aggressive,
}

impl WalkawayStyle {
    pub fn label(self) -> &'static str {
        match self {
            WalkawayStyle::Conservative => "Conservative",
            WalkawayStyle::Balanced => "Balanced",
            WalkawayStyle::Aggressive => "Aggressive",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            WalkawayStyle::Conservative => "Protects margin. May miss good deals.",
            WalkawayStyle::Balanced => "Default investor discipline.",
            WalkawayStyle::Aggressive => "Chases upside. Easier to overpay.",
        }
    }

    pub fn buffer_adjustment(self) -> i64 {
        match self {
            WalkawayStyle::Conservative => 24_000,
            WalkawayStyle::Balanced => 0,
            WalkawayStyle::Aggressive => -22_000,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ResearchReport {
    pub level: ResearchLevel,
}
