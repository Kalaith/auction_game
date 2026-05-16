use crate::model::{
    ActiveRenovation, CompletedUpgrade, ContractorTier, OwnedProperty, Player, UpgradeData,
};
use crate::sim::valuation::{current_value, round_to_1000};

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
    pub warning: String,
    pub note: String,
    pub is_overcapitalized: bool,
}

pub fn quote_renovation(
    owned: &OwnedProperty,
    upgrade: &UpgradeData,
    contractor: ContractorTier,
    market: &crate::model::MarketEvent,
) -> RenovationQuote {
    let synergy = synergy_multiplier(owned, upgrade);
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
    let permit_risk = permit_risk(upgrade, contractor);
    let projected_value = current_value(owned, market);
    let future_upgrade_spend = owned.upgrade_spend() + total_cost;
    let is_overcapitalized = future_upgrade_spend > round_to_1000(projected_value as f32 * 0.14)
        || total_cost > round_to_1000(value_boost as f32 * 1.25);
    let warning = renovation_warning(is_overcapitalized, permit_risk, total_cost, value_boost);
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
        warning,
        note,
        is_overcapitalized,
    }
}

pub fn start_upgrade_project(quote: &RenovationQuote, started_week: u32) -> ActiveRenovation {
    let delay_weeks = deterministic_delay_weeks(quote, started_week);
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

fn deterministic_delay_weeks(quote: &RenovationQuote, started_week: u32) -> u32 {
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
    if roll < quote.permit_risk {
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

fn permit_risk(upgrade: &UpgradeData, contractor: ContractorTier) -> i32 {
    let base = match upgrade.id.as_str() {
        "structural_repair" => 18,
        "kitchen_refresh" | "bathroom_upgrade" => 6,
        _ => 0,
    };
    (base + contractor.risk_modifier()).clamp(0, 40)
}

fn renovation_warning(
    is_overcapitalized: bool,
    permit_risk: i32,
    total_cost: i64,
    value_boost: i64,
) -> String {
    if is_overcapitalized {
        "Overcapitalisation risk: spend is outrunning likely value.".to_string()
    } else if permit_risk >= 18 {
        "Permit and delay risk: keep cash for holding costs.".to_string()
    } else if total_cost > value_boost {
        "Lifestyle upgrade: useful for emotion, weak on raw value.".to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Condition, MarketEvent, Property};
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
        let owned = OwnedProperty::new(test_property(), 430_000, 16_000, 52_000, 378_000, 530_000);
        let quote = quote_renovation(&owned, &upgrade, ContractorTier::Reliable, &test_market());
        let project = start_upgrade_project(&quote, 1);
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
