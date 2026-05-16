use crate::model::{MarketEvent, Property, ResearchLevel};
use crate::sim::valuation::{market_adjusted_value, round_down_to_increment, round_to_1000};

pub fn researched_value_range(
    property: &Property,
    market: &MarketEvent,
    level: ResearchLevel,
) -> (i64, i64) {
    let value = market_adjusted_value(property, market);
    let width = level.range_width();
    (
        round_to_1000(value as f32 * (1.0 - width)),
        round_to_1000(value as f32 * (1.0 + width)),
    )
}

pub fn recommended_walkaway(
    property: &Property,
    market: &MarketEvent,
    level: ResearchLevel,
) -> i64 {
    let value = market_adjusted_value(property, market);
    let base_buffer = match level {
        ResearchLevel::StreetScan => 42_000,
        ResearchLevel::AgentPack => 36_000,
        ResearchLevel::BuildingInspection => 30_000,
        ResearchLevel::FullDiligence => 26_000,
    };
    let defect_buffer = if material_defect_likely(property) {
        match level {
            ResearchLevel::StreetScan | ResearchLevel::AgentPack => 0,
            ResearchLevel::BuildingInspection => 18_000,
            ResearchLevel::FullDiligence => 28_000,
        }
    } else {
        0
    };
    round_down_to_increment(
        (value - base_buffer - defect_buffer).max(property.guide_price),
        10_000,
    )
}

pub fn risk_summary(property: &Property, level: ResearchLevel) -> String {
    let likely_defect = material_defect_likely(property);
    match level {
        ResearchLevel::StreetScan => "Risk unverified. Guide assumes no major defect.".to_string(),
        ResearchLevel::AgentPack => {
            if property.hidden_defect_risk >= 0.25 {
                "Agent pack flags elevated defect risk.".to_string()
            } else if property.hidden_defect_risk >= 0.15 {
                "Agent pack shows moderate defect risk.".to_string()
            } else {
                "Agent pack shows low defect risk.".to_string()
            }
        }
        ResearchLevel::BuildingInspection => {
            if likely_defect {
                "Inspector suspects a material repair item.".to_string()
            } else {
                "Inspector found no major structural warning.".to_string()
            }
        }
        ResearchLevel::FullDiligence => {
            if likely_defect {
                "Full diligence confirms a material defect allowance is needed.".to_string()
            } else {
                "Full diligence confirms no major hidden defect.".to_string()
            }
        }
    }
}

pub fn due_diligence_note(property: &Property, level: ResearchLevel) -> &'static str {
    match level {
        ResearchLevel::StreetScan => "You are bidding from public data and curb appeal.",
        ResearchLevel::AgentPack => {
            "Comparable sales are cleaner, but building risk is still fuzzy."
        }
        ResearchLevel::BuildingInspection if material_defect_likely(property) => {
            "The repair risk should lower your walk-away price."
        }
        ResearchLevel::BuildingInspection => {
            "The main defect risk is now priced with more confidence."
        }
        ResearchLevel::FullDiligence => "This is the cleanest pre-auction read available for now.",
    }
}

pub fn comparable_sale_value(property: &Property, market: &MarketEvent, index: usize) -> i64 {
    let adjusted = market_adjusted_value(property, market);
    let spread = match index {
        0 => -24_000,
        1 => 6_000,
        _ => 22_000,
    };
    adjusted + spread + property.id as i64 * 2_000
}

pub fn material_defect_likely(property: &Property) -> bool {
    property.hidden_defect_risk >= 0.28
        || (property.hidden_defect_risk >= 0.18 && property.id % 2 == 1)
}
