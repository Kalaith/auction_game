use super::*;
use crate::data::GameData;
use crate::model::{OwnedProperty, Property, ResearchLevel, WalkawayStyle};

fn sample_owned_property() -> (OwnedProperty, MarketEvent) {
    let data = GameData::load();
    let property = Property::from_template(&data.properties[0]);
    let owned = OwnedProperty::new(
        property,
        610_000,
        26_000,
        61_000,
        549_000,
        625_000,
        ResearchLevel::BuildingInspection,
        WalkawayStyle::Balanced,
    );
    (owned, data.market_events[0].clone())
}

#[test]
fn premium_marketing_adds_pressure_and_cost() {
    let (owned, market) = sample_owned_property();
    let budget = simulate_sale(&owned, &market, ReserveChoice::Fair, MarketingPlan::Budget);
    let premium = simulate_sale(&owned, &market, ReserveChoice::Fair, MarketingPlan::Premium);

    assert!(premium.demand_score >= budget.demand_score);
    assert_eq!(premium.marketing_cost, MarketingPlan::Premium.cost());
    assert!(premium.total_costs > budget.total_costs);
}
