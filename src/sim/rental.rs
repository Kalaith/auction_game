use crate::model::{Condition, DealArchetype, MarketEvent, Player, Property};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RentalSnapshot {
    pub gross_rent: i64,
    pub operating_cost: i64,
    pub net_income: i64,
}

pub fn weekly_rent_for(property: &Property, market: &MarketEvent) -> i64 {
    let base_yield = match property.deal_archetype {
        DealArchetype::RentalHold | DealArchetype::QuietBargain => 0.057,
        DealArchetype::RiskyFixer | DealArchetype::LandValuePlay => 0.052,
        DealArchetype::PrettyTrap | DealArchetype::HotSuburbFomo => 0.044,
        DealArchetype::RenovatorBait | DealArchetype::AuctionTrap => 0.049,
    };
    let condition_adjustment = match property.condition {
        Condition::Rough => -0.08,
        Condition::Tired => -0.03,
        Condition::Solid => 0.03,
        Condition::Premium => 0.07,
    };
    let demand_adjustment = (property.buyer_demand - 55) as f32 / 500.0;
    let market_adjustment = market.suburb_modifier(&property.suburb) * 0.25;
    round_to_ten(
        property.market_value as f32
            * base_yield
            * (1.0 + condition_adjustment + demand_adjustment + market_adjustment)
            / 52.0,
    )
}

pub fn leasing_cost(weekly_rent: i64) -> i64 {
    weekly_rent * 2
}

pub fn portfolio_rental_snapshot(player: &Player) -> RentalSnapshot {
    player
        .properties
        .iter()
        .filter(|owned| owned.is_leased)
        .fold(RentalSnapshot::default(), |mut total, owned| {
            let operating_cost = rental_operating_cost(owned.weekly_rent);
            total.gross_rent += owned.weekly_rent;
            total.operating_cost += operating_cost;
            total.net_income += owned.weekly_rent - operating_cost;
            total
        })
}

pub fn apply_rental_income(player: &mut Player) -> RentalSnapshot {
    let snapshot = portfolio_rental_snapshot(player);
    for owned in &mut player.properties {
        if owned.is_leased {
            let operating_cost = rental_operating_cost(owned.weekly_rent);
            owned.rent_received += owned.weekly_rent;
            owned.operating_spend += operating_cost;
        }
    }
    snapshot
}

fn rental_operating_cost(weekly_rent: i64) -> i64 {
    round_to_ten(weekly_rent as f32 * 0.12)
}

fn round_to_ten(value: f32) -> i64 {
    ((value / 10.0).round() as i64) * 10
}

#[cfg(test)]
mod tests;
