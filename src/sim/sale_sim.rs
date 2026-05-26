use crate::model::{DealArchetype, MarketEvent, OwnedProperty, ResearchLevel};
use crate::sim::valuation::{current_value, round_to_1000, sale_fees};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum ReserveChoice {
    Conservative,
    Fair,
    Ambitious,
}

impl ReserveChoice {
    pub fn label(self) -> &'static str {
        match self {
            ReserveChoice::Conservative => "Conservative",
            ReserveChoice::Fair => "Fair",
            ReserveChoice::Ambitious => "Ambitious",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SaleResult {
    pub property_address: String,
    pub reserve_choice: ReserveChoice,
    pub reserve_price: i64,
    pub purchase_price: i64,
    pub walkaway_price: i64,
    pub highest_bid: i64,
    pub sale_price: Option<i64>,
    pub bidder_count: i32,
    pub demand_score: i32,
    pub total_costs: i64,
    pub selling_fees: i64,
    pub profit: i64,
    pub purchase_discipline: String,
    pub research_quality: String,
    pub renovation_choice: String,
    pub sale_timing: String,
    pub next_time: String,
    pub reputation_delta: i32,
    pub reputation_reason: String,
    pub lesson: String,
}

pub fn simulate_sale(
    owned: &OwnedProperty,
    market: &MarketEvent,
    reserve_choice: ReserveChoice,
) -> SaleResult {
    let value = current_value(owned, market);
    let reserve_multiplier = match reserve_choice {
        ReserveChoice::Conservative => 0.94,
        ReserveChoice::Fair => 1.0,
        ReserveChoice::Ambitious => 1.08,
    };
    let reserve_price = round_to_1000(value as f32 * reserve_multiplier);
    let demand_score = demand_score(owned, market);
    let bidder_count = (2.0 + demand_score / 18.0).round().clamp(2.0, 8.0) as i32;
    let variance = deterministic_variance(owned.property.id, owned.upgrades.len());
    let competition_modifier = ((demand_score - 58.0) / 100.0).clamp(-0.10, 0.12);
    let highest_bid = round_to_1000(value as f32 * (0.98 + competition_modifier + variance));
    let sale_price = if highest_bid >= reserve_price {
        Some(highest_bid)
    } else {
        None
    };

    let selling_fees = sale_price.map(sale_fees).unwrap_or(0);
    let total_costs = owned.purchase_price
        + owned.purchase_fees
        + owned.upgrade_spend()
        + owned.holding_spend()
        + selling_fees;
    let profit = sale_price
        .map(|price| price - total_costs)
        .unwrap_or(-owned.holding_spend());

    let purchase_discipline = purchase_discipline(owned);
    let research_quality = research_quality(owned);
    let renovation_choice = renovation_choice(owned);
    let sale_timing = sale_timing(profit, sale_price, reserve_choice, owned, demand_score);
    let next_time = next_time(profit, sale_price, owned, reserve_choice);
    let reputation_delta = reputation_delta(profit, sale_price, owned);
    let reputation_reason = reputation_reason(reputation_delta, owned, profit);
    let lesson = sale_lesson(profit, sale_price, reserve_choice, owned);

    SaleResult {
        property_address: owned.property.address.clone(),
        reserve_choice,
        reserve_price,
        purchase_price: owned.purchase_price,
        walkaway_price: owned.walkaway_price,
        highest_bid,
        sale_price,
        bidder_count,
        demand_score: demand_score.round() as i32,
        total_costs,
        selling_fees,
        profit,
        purchase_discipline,
        research_quality,
        renovation_choice,
        sale_timing,
        next_time,
        reputation_delta,
        reputation_reason,
        lesson,
    }
}

fn demand_score(owned: &OwnedProperty, market: &MarketEvent) -> f32 {
    let mut score = owned.property.buyer_demand as f32;
    score += market.suburb_modifier(&owned.property.suburb) * 100.0;
    score += owned.property.appeal as f32 * 0.20;

    for upgrade in &owned.upgrades {
        score += upgrade.sale_emotion_boost as f32;
    }

    if owned.hidden_defect_discovered && !owned.has_defect_repair() {
        score -= 10.0;
    }

    match owned.property.deal_archetype {
        DealArchetype::HotSuburbFomo => score += 8.0,
        DealArchetype::QuietBargain => score -= 5.0,
        DealArchetype::PrettyTrap => score += 4.0,
        DealArchetype::LandValuePlay if owned.property.land_size >= 600 => score += 3.0,
        DealArchetype::RiskyFixer
            if owned.hidden_defect_discovered && !owned.has_defect_repair() =>
        {
            score -= 8.0
        }
        DealArchetype::RentalHold => score -= 3.0,
        _ => {}
    }

    score.clamp(20.0, 95.0)
}

fn deterministic_variance(property_id: usize, upgrade_count: usize) -> f32 {
    let seed = ((property_id as i32 * 37 + upgrade_count as i32 * 19 + 11) % 17) - 8;
    seed as f32 / 200.0
}

fn purchase_discipline(owned: &OwnedProperty) -> String {
    let over_walkaway = owned.purchase_price - owned.walkaway_price;
    if over_walkaway > 0 {
        format!(
            "Failed: bought {} over the {} walk-away plan.",
            crate::ui::format_money(over_walkaway),
            owned.walkaway_style.label()
        )
    } else {
        format!(
            "Held: bought within the {} walk-away plan.",
            owned.walkaway_style.label()
        )
    }
}

fn research_quality(owned: &OwnedProperty) -> String {
    let wrong_tool = match owned.property.deal_archetype {
        DealArchetype::RiskyFixer | DealArchetype::RenovatorBait => {
            owned.research_level < ResearchLevel::BuildingInspection
        }
        DealArchetype::AuctionTrap | DealArchetype::HotSuburbFomo => {
            owned.research_level < ResearchLevel::AgentPack
        }
        DealArchetype::LandValuePlay | DealArchetype::PrettyTrap => {
            owned.research_level < ResearchLevel::FullDiligence
        }
        DealArchetype::QuietBargain | DealArchetype::RentalHold => {
            owned.research_level < ResearchLevel::AgentPack
        }
    };

    if wrong_tool {
        return format!(
            "Wrong tool: {} needed deeper research for this {}.",
            owned.research_level.label(),
            owned.property.deal_archetype.label()
        );
    }

    match owned.research_level {
        ResearchLevel::StreetScan => {
            "Weak: public data left reserve and building risk fuzzy.".to_string()
        }
        ResearchLevel::AgentPack
            if owned.hidden_defect_discovered && !owned.has_defect_repair() =>
        {
            "Partial: reserve was clearer, but the defect still hurt resale.".to_string()
        }
        ResearchLevel::AgentPack => "Useful: reserve and comps improved the bid plan.".to_string(),
        ResearchLevel::BuildingInspection if owned.hidden_defect_discovered => {
            "Strong: the defect was visible before renovation decisions.".to_string()
        }
        ResearchLevel::BuildingInspection => {
            "Strong: building risk was checked before auction.".to_string()
        }
        ResearchLevel::FullDiligence => {
            "Excellent: value, reserve, and risk were all narrowed before bidding.".to_string()
        }
    }
}

fn renovation_choice(owned: &OwnedProperty) -> String {
    let spend = owned.upgrade_spend();
    let value_boost: i64 = owned
        .upgrades
        .iter()
        .map(|upgrade| upgrade.value_boost)
        .sum();
    if owned.hidden_defect_discovered && !owned.has_defect_repair() {
        "Weak: unrepaired defect kept buyers nervous.".to_string()
    } else if owned.hidden_defect_discovered && owned.has_defect_repair() {
        "Reasonable: repair protected resale value.".to_string()
    } else if spend == 0 {
        "Neutral: no renovation spend was added.".to_string()
    } else if spend > value_boost {
        "Thin: improvements helped appeal, but spend outran value.".to_string()
    } else if spend > round_to_1000(owned.property.market_value as f32 * 0.12) {
        "Risky: total renovation spend is close to overcapitalising.".to_string()
    } else {
        "Good: upgrades added value without swallowing the margin.".to_string()
    }
}

fn sale_timing(
    profit: i64,
    sale_price: Option<i64>,
    reserve_choice: ReserveChoice,
    owned: &OwnedProperty,
    demand_score: f32,
) -> String {
    if sale_price.is_none() {
        return "Poor: reserve and demand did not meet, so holding costs continue.".to_string();
    }
    if owned.weeks_held >= 8 && profit < 0 {
        "Late: holding costs had too long to compound.".to_string()
    } else if reserve_choice == ReserveChoice::Ambitious && demand_score < 58.0 {
        "Stretched: the reserve asked too much of a quiet market.".to_string()
    } else if reserve_choice == ReserveChoice::Conservative && demand_score >= 65.0 {
        "Good: conservative reserve let competition build.".to_string()
    } else {
        "Fair: timing matched the market without adding much edge.".to_string()
    }
}

fn next_time(
    profit: i64,
    sale_price: Option<i64>,
    owned: &OwnedProperty,
    reserve_choice: ReserveChoice,
) -> String {
    if owned.purchase_price > owned.walkaway_price {
        "Do not bid past walk-away unless new information changes the deal.".to_string()
    } else if owned.hidden_defect_discovered && !owned.has_defect_repair() {
        "Price the defect before auction, then repair or discount it deliberately.".to_string()
    } else if owned.upgrade_spend() > 0 && profit < 0 {
        "Check net renovation effect after holding costs, not just value uplift.".to_string()
    } else if sale_price.is_none() && reserve_choice == ReserveChoice::Ambitious {
        "Use a fairer reserve when demand is not strong enough to stretch.".to_string()
    } else {
        "Keep buying below your plan and let the market prove the upside.".to_string()
    }
}

fn reputation_delta(profit: i64, sale_price: Option<i64>, owned: &OwnedProperty) -> i32 {
    if sale_price.is_none() {
        return 0;
    }
    let disciplined = owned.purchase_price <= owned.walkaway_price;
    let overcapitalized =
        owned.upgrade_spend() > round_to_1000(owned.property.market_value as f32 * 0.14);
    if disciplined && profit >= 0 && !overcapitalized {
        2
    } else if disciplined && profit > -20_000 {
        1
    } else if !disciplined && profit < 0 {
        -2
    } else if overcapitalized && profit < 0 {
        -1
    } else if profit >= 0 {
        1
    } else {
        -1
    }
}

fn reputation_reason(delta: i32, owned: &OwnedProperty, profit: i64) -> String {
    if delta > 0 {
        if owned.purchase_price <= owned.walkaway_price {
            format!("+{} reputation for disciplined execution.", delta)
        } else {
            format!(
                "+{} reputation for salvaging profit despite a stretched buy.",
                delta
            )
        }
    } else if delta < 0 {
        format!(
            "{} reputation because the deal lost {}.",
            delta,
            crate::ui::format_money(profit.abs())
        )
    } else {
        "No reputation change. The deal is still unresolved.".to_string()
    }
}

fn sale_lesson(
    profit: i64,
    sale_price: Option<i64>,
    reserve_choice: ReserveChoice,
    owned: &OwnedProperty,
) -> String {
    if sale_price.is_none() {
        return "Your reserve protected the downside, but the market refused to meet it. Holding costs now matter."
            .to_string();
    }

    if profit < 0 {
        if owned.upgrade_spend() > 45_000 {
            "The renovation improved the house, but the spend outran the margin.".to_string()
        } else {
            "You won the deal, then the fees and holding costs took the profit away.".to_string()
        }
    } else if reserve_choice == ReserveChoice::Conservative && profit > 35_000 {
        "A conservative reserve built momentum and let bidders compete above the floor.".to_string()
    } else if profit > 0 {
        "You protected the margin and exited with a clean profit.".to_string()
    } else {
        "The deal landed close to break-even. Your next edge needs to come from buying better or bidding less."
            .to_string()
    }
}
