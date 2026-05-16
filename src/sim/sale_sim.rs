use crate::model::{MarketEvent, OwnedProperty};
use crate::sim::valuation::{current_value, round_to_1000, sale_fees};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug)]
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

    score.clamp(20.0, 95.0)
}

fn deterministic_variance(property_id: usize, upgrade_count: usize) -> f32 {
    let seed = ((property_id as i32 * 37 + upgrade_count as i32 * 19 + 11) % 17) - 8;
    seed as f32 / 200.0
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
