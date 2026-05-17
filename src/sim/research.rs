use crate::model::{DealArchetype, MarketEvent, Property, ResearchLevel, WalkawayStyle};
use crate::sim::valuation::{market_adjusted_value, round_down_to_increment, round_to_1000};

pub fn researched_value_range(
    property: &Property,
    market: &MarketEvent,
    level: ResearchLevel,
    reputation: i32,
) -> (i64, i64) {
    let value = market_adjusted_value(property, market);
    let archetype_noise = match property.deal_archetype {
        DealArchetype::RiskyFixer | DealArchetype::AuctionTrap
            if level < ResearchLevel::BuildingInspection =>
        {
            0.025
        }
        DealArchetype::PrettyTrap if level == ResearchLevel::StreetScan => 0.015,
        DealArchetype::QuietBargain if level >= ResearchLevel::AgentPack => -0.008,
        _ => 0.0,
    };
    let reputation_accuracy = (reputation.max(0) as f32 * 0.003).min(0.018);
    let width = (level.range_width() + archetype_noise - reputation_accuracy).max(0.018);
    (
        round_to_1000(value as f32 * (1.0 - width)),
        round_to_1000(value as f32 * (1.0 + width)),
    )
}

pub fn recommended_walkaway(
    property: &Property,
    market: &MarketEvent,
    level: ResearchLevel,
    style: WalkawayStyle,
    reputation: i32,
) -> i64 {
    let value = market_adjusted_value(property, market);
    let base_buffer = match level {
        ResearchLevel::StreetScan => 42_000,
        ResearchLevel::AgentPack => 36_000,
        ResearchLevel::BuildingInspection => 30_000,
        ResearchLevel::FullDiligence => 26_000,
    };
    let archetype_buffer = archetype_walkaway_buffer(property, level);
    let reputation_buffer_relief = i64::from(reputation.max(0)).min(6) * 2_000;
    let defect_buffer = if material_defect_likely(property) {
        match level {
            ResearchLevel::StreetScan | ResearchLevel::AgentPack => {
                if property.deal_archetype == DealArchetype::RiskyFixer {
                    18_000
                } else {
                    0
                }
            }
            ResearchLevel::BuildingInspection => {
                if property.deal_archetype == DealArchetype::RiskyFixer {
                    30_000
                } else {
                    18_000
                }
            }
            ResearchLevel::FullDiligence => {
                if property.deal_archetype == DealArchetype::RiskyFixer {
                    36_000
                } else {
                    28_000
                }
            }
        }
    } else {
        0
    };
    round_down_to_increment(
        (value - base_buffer - archetype_buffer - defect_buffer - style.buffer_adjustment()
            + reputation_buffer_relief)
            .max(property.guide_price),
        10_000,
    )
}

pub fn research_cost(level: ResearchLevel, reputation: i32) -> i64 {
    let base = level.cost();
    if base == 0 {
        return 0;
    }
    let discount = (reputation.max(0) as f32 * 0.05).min(0.30);
    round_down_to_increment((base as f32 * (1.0 - discount)) as i64, 100).max(500)
}

pub fn research_question(level: ResearchLevel) -> &'static str {
    match level {
        ResearchLevel::StreetScan => "What is the rough value and demand?",
        ResearchLevel::AgentPack => "What reserve and buyer interest should you expect?",
        ResearchLevel::BuildingInspection => "What defect or repair bill could kill the margin?",
        ResearchLevel::FullDiligence => "How wrong could your walk-away number still be?",
    }
}

pub fn research_takeaway(
    property: &Property,
    market: &MarketEvent,
    level: ResearchLevel,
) -> String {
    match level {
        ResearchLevel::StreetScan => format!(
            "{}. Public data says {} demand with a wide value band.",
            property.deal_archetype.lesson(),
            demand_word(property.buyer_demand)
        ),
        ResearchLevel::AgentPack => format!(
            "Reserve likely sits near {}. Buyer interest looks {} against this guide.",
            crate::ui::format_money(estimate_reserve(property, market, level, 0)),
            demand_word(
                property.buyer_demand + (market.suburb_modifier(&property.suburb) * 100.0) as i32
            )
        ),
        ResearchLevel::BuildingInspection => {
            if material_defect_likely(property) {
                format!(
                    "Building risk is real. Hold back at least {} for repairs.",
                    crate::ui::format_money(defect_allowance(property, level))
                )
            } else {
                "No major defect surfaced. The risk is now price discipline, not repair shock."
                    .to_string()
            }
        }
        ResearchLevel::FullDiligence => format!(
            "The thesis is now testable: {} Avoid the trap to {}.",
            property.best_strategy,
            property.deal_archetype.temptation()
        ),
    }
}

pub fn estimate_reserve(
    property: &Property,
    market: &MarketEvent,
    level: ResearchLevel,
    reputation: i32,
) -> i64 {
    let confidence_gap = match level {
        ResearchLevel::StreetScan => -20_000,
        ResearchLevel::AgentPack => -8_000,
        ResearchLevel::BuildingInspection => -4_000,
        ResearchLevel::FullDiligence => 0,
    };
    let archetype_gap = match property.deal_archetype {
        DealArchetype::AuctionTrap if level == ResearchLevel::StreetScan => -28_000,
        DealArchetype::AuctionTrap if level == ResearchLevel::AgentPack => -8_000,
        DealArchetype::PrettyTrap if level < ResearchLevel::FullDiligence => 10_000,
        DealArchetype::QuietBargain if level >= ResearchLevel::AgentPack => -5_000,
        _ => 0,
    };
    let reputation_correction = i64::from(reputation.max(0)).min(6) * 2_000;
    let market_gap = round_to_1000(market.suburb_modifier(&property.suburb) * 30_000.0);
    round_down_to_increment(
        property.reserve_price
            + confidence_gap
            + archetype_gap
            + market_gap
            + reputation_correction,
        5_000,
    )
}

pub fn defect_allowance(property: &Property, level: ResearchLevel) -> i64 {
    let base =
        round_to_1000(property.market_value as f32 * property.condition.defect_penalty_rate());
    match level {
        ResearchLevel::StreetScan => round_to_1000(base as f32 * 0.55),
        ResearchLevel::AgentPack => round_to_1000(base as f32 * 0.75),
        ResearchLevel::BuildingInspection => base,
        ResearchLevel::FullDiligence => round_to_1000(base as f32 * 1.12),
    }
}

pub fn risk_summary(property: &Property, level: ResearchLevel) -> String {
    let likely_defect = material_defect_likely(property);
    match level {
        ResearchLevel::StreetScan => format!(
            "Risk unverified. The listing wants you to {}.",
            property.deal_archetype.temptation()
        ),
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

pub fn research_fit_summary(property: &Property, level: ResearchLevel) -> &'static str {
    match property.deal_archetype {
        DealArchetype::RiskyFixer | DealArchetype::RenovatorBait => {
            if level >= ResearchLevel::BuildingInspection {
                "Research fit: correct tool for repair risk."
            } else {
                "Research gap: building risk is still the main blind spot."
            }
        }
        DealArchetype::AuctionTrap | DealArchetype::HotSuburbFomo => {
            if level >= ResearchLevel::AgentPack {
                "Research fit: reserve and bidder pressure are clearer."
            } else {
                "Research gap: guide price may be bait."
            }
        }
        DealArchetype::LandValuePlay => {
            if level >= ResearchLevel::FullDiligence {
                "Research fit: land thesis has been stress-tested."
            } else {
                "Research gap: land value needs deeper confirmation."
            }
        }
        DealArchetype::PrettyTrap => {
            if level >= ResearchLevel::FullDiligence {
                "Research fit: safety premium is priced."
            } else {
                "Research gap: safe-looking can still mean overpriced."
            }
        }
        DealArchetype::QuietBargain | DealArchetype::RentalHold => {
            if level >= ResearchLevel::AgentPack {
                "Research fit: quiet numbers have enough confirmation."
            } else {
                "Research gap: cheap-looking may just be unloved."
            }
        }
    }
}

pub fn due_diligence_note(property: &Property, level: ResearchLevel) -> &'static str {
    match level {
        ResearchLevel::StreetScan => "You are bidding from public data and curb appeal.",
        ResearchLevel::AgentPack => {
            "Reserve and comparable sales are cleaner, but building risk is still fuzzy."
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

fn demand_word(score: i32) -> &'static str {
    if score >= 72 {
        "hot"
    } else if score >= 55 {
        "steady"
    } else {
        "soft"
    }
}

fn archetype_walkaway_buffer(property: &Property, level: ResearchLevel) -> i64 {
    match property.deal_archetype {
        DealArchetype::PrettyTrap => 18_000,
        DealArchetype::HotSuburbFomo => 14_000,
        DealArchetype::AuctionTrap if level < ResearchLevel::AgentPack => 24_000,
        DealArchetype::AuctionTrap => 10_000,
        DealArchetype::RenovatorBait => 12_000,
        DealArchetype::RiskyFixer if level < ResearchLevel::BuildingInspection => 16_000,
        DealArchetype::LandValuePlay if level >= ResearchLevel::FullDiligence => -8_000,
        DealArchetype::QuietBargain if level >= ResearchLevel::AgentPack => -10_000,
        DealArchetype::RentalHold => -6_000,
        _ => 0,
    }
}
