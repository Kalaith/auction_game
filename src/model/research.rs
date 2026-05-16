#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug)]
pub struct ResearchReport {
    pub level: ResearchLevel,
}
