use crate::model::{MarketEvent, OwnedProperty, Player, Property};

pub const DEPOSIT_RATE: f32 = 0.12;
pub const BUYER_FEE_RATE: f32 = 0.038;
pub const SELLING_FEE_RATE: f32 = 0.022;

pub fn market_adjusted_value(property: &Property, market: &MarketEvent) -> i64 {
    let suburb_modifier = market.suburb_modifier(&property.suburb);
    let condition_modifier = match property.condition {
        crate::model::Condition::Rough | crate::model::Condition::Tired => {
            market.renovator_modifier
        }
        crate::model::Condition::Solid | crate::model::Condition::Premium => 0.0,
    };
    round_to_1000(property.market_value as f32 * (1.0 + suburb_modifier + condition_modifier))
}

pub fn estimated_value_range(property: &Property, market: &MarketEvent) -> (i64, i64) {
    let value = market_adjusted_value(property, market);
    (
        round_to_1000(value as f32 * 0.94),
        round_to_1000(value as f32 * 1.06),
    )
}

pub fn purchase_fees(price: i64) -> i64 {
    round_to_1000(price as f32 * BUYER_FEE_RATE)
}

pub fn deposit(price: i64) -> i64 {
    round_to_1000(price as f32 * DEPOSIT_RATE)
}

pub fn cash_needed_to_settle(price: i64) -> i64 {
    deposit(price) + purchase_fees(price)
}

pub fn current_value(owned: &OwnedProperty, market: &MarketEvent) -> i64 {
    let mut value = market_adjusted_value(&owned.property, market);
    let mut appeal_boost = 0;

    for upgrade in &owned.upgrades {
        value += upgrade.value_boost;
        appeal_boost += upgrade.appeal_boost;
    }

    if owned.hidden_defect_discovered && !owned.has_defect_repair() {
        let penalty = owned.property.condition.defect_penalty_rate();
        value = round_to_1000(value as f32 * (1.0 - penalty));
    }

    let appeal_modifier = (appeal_boost as f32 / 100.0).min(0.18);
    round_to_1000(value as f32 * (1.0 + appeal_modifier))
}

pub fn sale_fees(sale_price: i64) -> i64 {
    round_to_1000(sale_price as f32 * SELLING_FEE_RATE)
}

pub fn projected_purchase_margin(
    property: &Property,
    purchase_price: i64,
    market: &MarketEvent,
) -> i64 {
    let resale_value = market_adjusted_value(property, market);
    let fees = purchase_fees(purchase_price) + sale_fees(resale_value);
    resale_value - purchase_price - fees
}

pub fn net_worth(player: &Player, market: &MarketEvent) -> i64 {
    let portfolio_value: i64 = player
        .properties
        .iter()
        .map(|owned| current_value(owned, market))
        .sum();
    player.cash + portfolio_value - player.debt
}

pub fn round_to_1000(value: f32) -> i64 {
    ((value / 1000.0).round() as i64) * 1000
}

pub fn round_down_to_increment(value: i64, increment: i64) -> i64 {
    (value / increment) * increment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purchase_cash_needed_includes_deposit_and_fees() {
        assert_eq!(deposit(600_000), 72_000);
        assert_eq!(purchase_fees(600_000), 23_000);
        assert_eq!(cash_needed_to_settle(600_000), 95_000);
    }
}
