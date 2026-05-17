use crate::model::{
    ActiveRenovation, CompletedUpgrade, ContractorTier, DealArchetype, OwnedProperty, Player,
    UpgradeData,
};
use crate::sim::valuation::{current_value, round_to_1000};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenovationVerdict {
    GoodRoi,
    ThinRoi,
    Overcapitalising,
    RepairsDefect,
    ImprovesSaleEmotion,
    BadTiming,
}

impl RenovationVerdict {
    pub fn label(self) -> &'static str {
        match self {
            RenovationVerdict::GoodRoi => "Good ROI",
            RenovationVerdict::ThinRoi => "Thin ROI",
            RenovationVerdict::Overcapitalising => "Overcapitalising",
            RenovationVerdict::RepairsDefect => "Repairs defect",
            RenovationVerdict::ImprovesSaleEmotion => "Improves sale emotion",
            RenovationVerdict::BadTiming => "Bad timing",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenovationDiagnosis {
    pub summary: String,
    pub best_move: String,
    pub is_warning: bool,
}

#[derive(Clone, Debug)]
pub struct RenovationQuote {
    pub upgrade_id: String,
    pub upgrade_name: String,
    pub contractor: ContractorTier,
    pub total_cost: i64,
    pub holding_cost: i64,
    pub holding_weeks: u32,
    pub value_boost: i64,
    pub appeal_boost: i32,
    pub sale_emotion_boost: i32,
    pub removes_defect: bool,
    pub permit_risk: i32,
    pub net_effect: i64,
    pub verdict: RenovationVerdict,
    pub warning: String,
    pub note: String,
    pub is_overcapitalized: bool,
}

pub fn quote_renovation(
    owned: &OwnedProperty,
    upgrade: &UpgradeData,
    contractor: ContractorTier,
    market: &crate::model::MarketEvent,
    reputation: i32,
) -> RenovationQuote {
    let synergy = synergy_multiplier(owned, upgrade) * archetype_upgrade_multiplier(owned, upgrade);
    let mut value_boost =
        round_to_1000(upgrade.value_boost as f32 * contractor.value_multiplier() * synergy);
    let mut appeal_boost = upgrade.appeal_boost;
    let mut sale_emotion_boost = upgrade.sale_emotion_boost;

    if upgrade.removes_defect && owned.hidden_defect_discovered && !owned.has_defect_repair() {
        value_boost += round_to_1000(owned.property.market_value as f32 * 0.045);
        appeal_boost += 4;
    }

    if upgrade.id == "staging" && owned.has_upgrade("landscaping") {
        sale_emotion_boost += 4;
    }
    if upgrade.id == "landscaping" && owned.has_upgrade("staging") {
        sale_emotion_boost += 3;
    }

    let base_weeks = base_weeks(upgrade);
    let holding_weeks = (base_weeks as i32 + contractor.week_modifier()).max(1) as u32;
    let total_cost = round_to_1000(upgrade.cost as f32 * contractor.cost_multiplier());
    let holding_cost = i64::from(holding_weeks) * owned.property.holding_cost_per_week;
    let permit_risk = permit_risk(upgrade, contractor, reputation);
    let projected_value = current_value(owned, market);
    let future_upgrade_spend = owned.upgrade_spend() + total_cost;
    let is_overcapitalized = future_upgrade_spend > round_to_1000(projected_value as f32 * 0.14)
        || total_cost > round_to_1000(value_boost as f32 * 1.25);
    let net_effect = value_boost - total_cost - holding_cost;
    let verdict = renovation_verdict(owned, upgrade, net_effect, is_overcapitalized);
    let warning = renovation_warning(verdict, permit_risk, net_effect);
    let note = renovation_note(upgrade, contractor, synergy);

    RenovationQuote {
        upgrade_id: upgrade.id.clone(),
        upgrade_name: upgrade.name.clone(),
        contractor,
        total_cost,
        holding_cost,
        holding_weeks,
        value_boost,
        appeal_boost,
        sale_emotion_boost,
        removes_defect: upgrade.removes_defect,
        permit_risk,
        net_effect,
        verdict,
        warning,
        note,
        is_overcapitalized,
    }
}

pub fn start_upgrade_project(
    quote: &RenovationQuote,
    started_week: u32,
    reputation: i32,
) -> ActiveRenovation {
    let delay_weeks = deterministic_delay_weeks(quote, started_week, reputation);
    let weeks_total = quote.holding_weeks + delay_weeks;
    let note = if delay_weeks > 0 {
        format!("{} Permit delay added {} week.", quote.note, delay_weeks)
    } else {
        quote.note.clone()
    };

    ActiveRenovation {
        upgrade_id: quote.upgrade_id.clone(),
        upgrade_name: quote.upgrade_name.clone(),
        contractor: quote.contractor,
        total_cost: quote.total_cost,
        value_boost: quote.value_boost,
        appeal_boost: quote.appeal_boost,
        sale_emotion_boost: quote.sale_emotion_boost,
        removes_defect: quote.removes_defect,
        weeks_total,
        weeks_remaining: weeks_total,
        permit_risk: quote.permit_risk,
        delay_weeks,
        note,
    }
}

pub fn progress_player_renovations(player: &mut Player) -> Vec<String> {
    player
        .properties
        .iter_mut()
        .filter_map(progress_renovation)
        .collect()
}

pub fn diagnose_property(
    owned: &OwnedProperty,
    market: &crate::model::MarketEvent,
) -> RenovationDiagnosis {
    let estimate = current_value(owned, market);
    let position = estimate
        - owned.purchase_price
        - owned.purchase_fees
        - owned.upgrade_spend()
        - owned.holding_spend();

    if owned.purchase_price > owned.walkaway_price {
        RenovationDiagnosis {
            summary: "You bought above plan. Renovation cannot fully save a bad entry price."
                .to_string(),
            best_move: treatment_plan(owned, "repair only real defects, then sell quickly"),
            is_warning: true,
        }
    } else if owned.hidden_defect_discovered && !owned.has_defect_repair() {
        RenovationDiagnosis {
            summary: "Structural risk is now the deal's main illness.".to_string(),
            best_move: treatment_plan(owned, "repair the defect before asking buyers to compete"),
            is_warning: true,
        }
    } else if position >= 45_000 {
        RenovationDiagnosis {
            summary: "Strong purchase. You have margin to choose small appeal upgrades."
                .to_string(),
            best_move: treatment_plan(owned, "use cosmetic work only where it solves the thesis"),
            is_warning: false,
        }
    } else if owned.upgrade_spend() > round_to_1000(estimate as f32 * 0.12) {
        RenovationDiagnosis {
            summary: "Spend is close to swallowing the remaining margin.".to_string(),
            best_move: treatment_plan(owned, "stop upgrading and test the sale market"),
            is_warning: true,
        }
    } else {
        RenovationDiagnosis {
            summary: "The deal is thin. Holding costs matter as much as upgrades.".to_string(),
            best_move: treatment_plan(
                owned,
                "choose the cheapest action that removes a buyer objection",
            ),
            is_warning: false,
        }
    }
}

fn progress_renovation(owned: &mut OwnedProperty) -> Option<String> {
    let mut project = owned.active_renovation.take()?;
    project.weeks_remaining = project.weeks_remaining.saturating_sub(1);

    if project.weeks_remaining > 0 {
        owned.active_renovation = Some(project);
        return None;
    }

    let message = format!(
        "{} finished at {} after {} week{}.",
        project.upgrade_name,
        owned.property.address,
        project.weeks_total,
        if project.weeks_total == 1 { "" } else { "s" }
    );
    owned.upgrades.push(complete_project(&project));
    Some(message)
}

fn complete_project(project: &ActiveRenovation) -> CompletedUpgrade {
    CompletedUpgrade {
        upgrade_id: project.upgrade_id.clone(),
        name: project.upgrade_name.clone(),
        contractor: project.contractor,
        actual_cost: project.total_cost,
        value_boost: project.value_boost,
        appeal_boost: project.appeal_boost,
        sale_emotion_boost: project.sale_emotion_boost,
        removes_defect: project.removes_defect,
        weeks_taken: project.weeks_total,
        note: project.note.clone(),
    }
}

fn synergy_multiplier(owned: &OwnedProperty, upgrade: &UpgradeData) -> f32 {
    let family_home = owned.property.bedrooms >= 3 && owned.property.buyer_demand >= 65;
    match upgrade.id.as_str() {
        "kitchen_refresh" if family_home => 1.16,
        "bathroom_upgrade" if owned.has_upgrade("kitchen_refresh") => 1.10,
        "landscaping" if owned.property.land_size >= 580 => 1.12,
        "staging" if owned.property.appeal >= 70 => 1.12,
        _ => 1.0,
    }
}

fn archetype_upgrade_multiplier(owned: &OwnedProperty, upgrade: &UpgradeData) -> f32 {
    match owned.property.deal_archetype {
        DealArchetype::PrettyTrap
            if matches!(upgrade.id.as_str(), "kitchen_refresh" | "bathroom_upgrade") =>
        {
            0.76
        }
        DealArchetype::PrettyTrap if upgrade.id == "staging" => 1.08,
        DealArchetype::LandValuePlay
            if matches!(upgrade.id.as_str(), "kitchen_refresh" | "staging") =>
        {
            0.66
        }
        DealArchetype::LandValuePlay if upgrade.id == "landscaping" => 1.14,
        DealArchetype::RiskyFixer if upgrade.removes_defect => 1.22,
        DealArchetype::RenovatorBait
            if matches!(upgrade.id.as_str(), "kitchen_refresh" | "bathroom_upgrade") =>
        {
            0.82
        }
        DealArchetype::HotSuburbFomo if upgrade.id == "staging" => 1.18,
        DealArchetype::QuietBargain if matches!(upgrade.id.as_str(), "paint_clean" | "staging") => {
            1.10
        }
        DealArchetype::RentalHold if upgrade.id == "structural_repair" => 1.12,
        _ => 1.0,
    }
}

fn deterministic_delay_weeks(quote: &RenovationQuote, started_week: u32, reputation: i32) -> u32 {
    if quote.permit_risk == 0 {
        return 0;
    }

    let tier_seed = match quote.contractor {
        ContractorTier::Budget => 23,
        ContractorTier::Reliable => 11,
        ContractorTier::Premium => 3,
    };
    let id_seed = quote
        .upgrade_id
        .bytes()
        .fold(0_u32, |total, byte| total + u32::from(byte));
    let roll = ((id_seed + started_week * 17 + tier_seed) % 100) as i32;
    let adjusted_risk = (quote.permit_risk - reputation.max(0) * 2).max(0);
    if roll < adjusted_risk {
        1
    } else {
        0
    }
}

fn base_weeks(upgrade: &UpgradeData) -> u32 {
    match upgrade.id.as_str() {
        "paint_clean" | "staging" => 1,
        "landscaping" => 2,
        "kitchen_refresh" | "bathroom_upgrade" => 3,
        "structural_repair" => 5,
        _ => 3,
    }
}

fn permit_risk(upgrade: &UpgradeData, contractor: ContractorTier, reputation: i32) -> i32 {
    let base = match upgrade.id.as_str() {
        "structural_repair" => 18,
        "kitchen_refresh" | "bathroom_upgrade" => 6,
        _ => 0,
    };
    (base + contractor.risk_modifier() - reputation.max(0) * 2).clamp(0, 40)
}

fn renovation_verdict(
    owned: &OwnedProperty,
    upgrade: &UpgradeData,
    net_effect: i64,
    is_overcapitalized: bool,
) -> RenovationVerdict {
    if upgrade.removes_defect && owned.hidden_defect_discovered && !owned.has_defect_repair() {
        RenovationVerdict::RepairsDefect
    } else if is_overcapitalized || net_effect < -8_000 {
        RenovationVerdict::Overcapitalising
    } else if net_effect < 5_000 {
        RenovationVerdict::ThinRoi
    } else if upgrade.sale_emotion_boost >= 8 && upgrade.value_boost < upgrade.cost {
        RenovationVerdict::ImprovesSaleEmotion
    } else if owned.weeks_held >= 8 && upgrade.cost >= 20_000 {
        RenovationVerdict::BadTiming
    } else {
        RenovationVerdict::GoodRoi
    }
}

fn renovation_warning(verdict: RenovationVerdict, permit_risk: i32, net_effect: i64) -> String {
    if verdict == RenovationVerdict::Overcapitalising {
        "Overcapitalisation risk: spend is outrunning likely value.".to_string()
    } else if permit_risk >= 18 {
        "Permit and delay risk: keep cash for holding costs.".to_string()
    } else if verdict == RenovationVerdict::ThinRoi {
        format!(
            "Thin ROI: expected net effect is only {}.",
            crate::ui::format_money(net_effect)
        )
    } else if verdict == RenovationVerdict::RepairsDefect {
        "Repairs defect: not pretty, but it protects resale confidence.".to_string()
    } else if verdict == RenovationVerdict::ImprovesSaleEmotion {
        "Sale emotion play: weak raw value, useful for auction energy.".to_string()
    } else if verdict == RenovationVerdict::BadTiming {
        "Bad timing: another large job lets holding costs keep biting.".to_string()
    } else {
        "Numbers look reasonable if the auction demand holds.".to_string()
    }
}

fn renovation_note(upgrade: &UpgradeData, contractor: ContractorTier, synergy: f32) -> String {
    if synergy > 1.05 {
        format!("{} creates a synergy on this property.", upgrade.name)
    } else {
        format!(
            "{} contractor: {}. {}",
            contractor.label(),
            upgrade.name,
            upgrade.description
        )
    }
}

fn treatment_plan(owned: &OwnedProperty, default: &str) -> String {
    match owned.property.deal_archetype {
        DealArchetype::RiskyFixer => format!("{default}; structural repair beats cosmetics."),
        DealArchetype::PrettyTrap => format!("{default}; avoid paying again for polish."),
        DealArchetype::LandValuePlay => format!("{default}; protect land value, avoid interiors."),
        DealArchetype::HotSuburbFomo => {
            format!("{default}; staging works only if resale momentum is still hot.")
        }
        DealArchetype::QuietBargain => format!("{default}; cheap presentation is enough."),
        DealArchetype::RenovatorBait => format!("{default}; cap spend before kitchens and baths."),
        DealArchetype::RentalHold => {
            format!("{default}; protect condition before chasing flip appeal.")
        }
        DealArchetype::AuctionTrap => {
            format!("{default}; buying discipline was the main treatment.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Condition, DealArchetype, MarketEvent, Property, ResearchLevel, WalkawayStyle,
    };
    use std::collections::HashMap;

    fn test_property() -> Property {
        Property {
            id: 7,
            address: "12 Test Street".to_string(),
            suburb: "Ridgefield".to_string(),
            bedrooms: 3,
            bathrooms: 1,
            condition: Condition::Rough,
            land_size: 640,
            market_value: 560_000,
            guide_price: 470_000,
            reserve_price: 505_000,
            appeal: 48,
            renovation_potential: 84,
            hidden_defect_risk: 0.32,
            holding_cost_per_week: 2_000,
            buyer_demand: 68,
            deal_archetype: DealArchetype::RiskyFixer,
            thesis: "Test thesis".to_string(),
            main_risk: "Test risk".to_string(),
            best_strategy: "Test strategy".to_string(),
            bad_strategy: "Test mistake".to_string(),
            notes: "Test fixture".to_string(),
        }
    }

    fn test_market() -> MarketEvent {
        MarketEvent {
            title: "Test".to_string(),
            items: vec![],
            suburb_modifiers: HashMap::new(),
            renovator_modifier: 0.04,
            buyer_budget_modifier: 0.0,
            strategy_effect: "Test strategy effect".to_string(),
        }
    }

    #[test]
    fn renovation_project_completes_after_weekly_progress() {
        let upgrade = UpgradeData {
            id: "paint_clean".to_string(),
            name: "Paint and Clean".to_string(),
            cost: 9_000,
            value_boost: 18_000,
            appeal_boost: 8,
            sale_emotion_boost: 4,
            removes_defect: false,
            description: "Presentation lift.".to_string(),
        };
        let owned = OwnedProperty::new(
            test_property(),
            430_000,
            16_000,
            52_000,
            378_000,
            530_000,
            ResearchLevel::BuildingInspection,
            WalkawayStyle::Balanced,
        );
        let quote = quote_renovation(
            &owned,
            &upgrade,
            ContractorTier::Reliable,
            &test_market(),
            0,
        );
        let project = start_upgrade_project(&quote, 1, 0);
        let mut player = Player::new();
        player.properties.push(owned);
        player.properties[0].active_renovation = Some(project.clone());

        for _ in 1..project.weeks_total {
            assert!(progress_player_renovations(&mut player).is_empty());
            assert!(player.properties[0].active_renovation.is_some());
        }

        let messages = progress_player_renovations(&mut player);
        assert_eq!(messages.len(), 1);
        assert!(player.properties[0].active_renovation.is_none());
        assert!(player.properties[0].has_upgrade("paint_clean"));
    }
}
