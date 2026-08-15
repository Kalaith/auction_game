use super::*;
use crate::model::DealArchetype;
use crate::model::Property;
use crate::sim::rental::weekly_rent_for;
use std::collections::HashSet;

#[test]
fn authored_market_has_unique_properties_and_full_rotation() {
    let data = GameData::load();
    let ids: HashSet<_> = data.properties.iter().map(|property| property.id).collect();

    assert_eq!(data.properties.len(), 12);
    assert_eq!(ids.len(), data.properties.len());
    assert!(data.market_events.len() >= 6);
    assert!(
        data.properties
            .iter()
            .filter(|property| property.deal_archetype == DealArchetype::RentalHold)
            .count()
            >= 2
    );
}

#[test]
fn weekly_property_costs_stay_in_the_rebalanced_band() {
    let data = GameData::load();

    assert!(data
        .properties
        .iter()
        .all(|property| (90..=220).contains(&property.holding_cost_per_week)));
}

#[test]
fn starter_market_contains_multiple_high_yield_choices() {
    let data = GameData::load();
    let market = &data.market_events[0];
    let high_yield_count = data
        .properties
        .iter()
        .map(Property::from_template)
        .filter(|property| {
            weekly_rent_for(property, market) as f32 * 52.0 / property.guide_price as f32 >= 0.05
        })
        .count();

    assert!(high_yield_count >= 4);
}
