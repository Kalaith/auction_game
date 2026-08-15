use super::*;
use crate::model::{OwnedProperty, Property, PropertyTemplate};
use std::collections::HashMap;

fn property() -> Property {
    Property::from_template(&PropertyTemplate {
        id: 1,
        address: "1 Test Street".to_string(),
        suburb: "Westport".to_string(),
        bedrooms: 3,
        bathrooms: 1,
        condition: Condition::Tired,
        land_size: 500,
        market_value: 500_000,
        guide_price: 450_000,
        reserve_price: 470_000,
        appeal: 50,
        renovation_potential: 60,
        hidden_defect_risk: 0.1,
        holding_cost_per_week: 120,
        buyer_demand: 55,
        deal_archetype: DealArchetype::RentalHold,
        thesis: String::new(),
        main_risk: String::new(),
        best_strategy: String::new(),
        bad_strategy: String::new(),
        notes: String::new(),
    })
}

fn market() -> MarketEvent {
    MarketEvent {
        title: String::new(),
        items: Vec::new(),
        suburb_modifiers: HashMap::new(),
        renovator_modifier: 0.0,
        buyer_budget_modifier: 0.0,
        strategy_effect: String::new(),
    }
}

#[test]
fn rental_hold_produces_plausible_weekly_rent() {
    let rent = weekly_rent_for(&property(), &market());
    assert!((500..=560).contains(&rent));
}

#[test]
fn leased_property_records_income_and_operating_cost() {
    let property = property();
    let mut owned = OwnedProperty::new(
        property.clone(),
        470_000,
        18_000,
        56_000,
        414_000,
        480_000,
        crate::model::ResearchLevel::StreetScan,
        crate::model::WalkawayStyle::Balanced,
    );
    owned.is_leased = true;
    owned.weekly_rent = weekly_rent_for(&property, &market());
    let mut player = Player::new();
    player.properties.push(owned);

    let snapshot = apply_rental_income(&mut player);

    assert!(snapshot.net_income > 0);
    assert_eq!(player.properties[0].rent_received, snapshot.gross_rent);
    assert_eq!(
        player.properties[0].operating_spend,
        snapshot.operating_cost
    );
}
