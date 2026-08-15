use super::*;
use crate::model::{
    Condition, DealArchetype, OwnedProperty, Property, ResearchLevel, WalkawayStyle,
};

fn leased_property() -> OwnedProperty {
    let property = Property {
        id: 1,
        address: "1 Test Street".to_string(),
        suburb: "Westport".to_string(),
        bedrooms: 3,
        bathrooms: 1,
        condition: Condition::Solid,
        land_size: 450,
        market_value: 500_000,
        guide_price: 450_000,
        reserve_price: 470_000,
        appeal: 60,
        renovation_potential: 50,
        hidden_defect_risk: 0.1,
        holding_cost_per_week: 110,
        buyer_demand: 60,
        deal_archetype: DealArchetype::RentalHold,
        thesis: String::new(),
        main_risk: String::new(),
        best_strategy: String::new(),
        bad_strategy: String::new(),
        notes: String::new(),
    };
    let mut owned = OwnedProperty::new(
        property,
        470_000,
        18_000,
        56_000,
        414_000,
        480_000,
        ResearchLevel::StreetScan,
        WalkawayStyle::Balanced,
    );
    owned.is_leased = true;
    owned.weekly_rent = 550;
    owned
}

#[test]
fn issue_arrives_on_the_disclosed_property_schedule() {
    let mut player = Player::new();
    let mut owned = leased_property();
    owned.weeks_held = next_maintenance_week(&owned);
    player.properties.push(owned);

    let notices = trigger_due_maintenance(&mut player);

    assert_eq!(notices.len(), 1);
    assert!(player.properties[0].maintenance_issue.is_some());
    assert!(effective_weekly_rent(&player.properties[0]) < 550);
}

#[test]
fn repair_restores_rent_and_schedules_a_later_check() {
    let mut owned = leased_property();
    owned.weeks_held = next_maintenance_week(&owned);
    let mut player = Player::new();
    player.properties.push(owned);
    trigger_due_maintenance(&mut player);
    let reduced_rent = effective_weekly_rent(&player.properties[0]);

    let cost = repair_maintenance(&mut player.properties[0]);

    assert!(cost > 0);
    assert_eq!(effective_weekly_rent(&player.properties[0]), 550);
    assert!(next_maintenance_week(&player.properties[0]) > player.properties[0].weeks_held);
    assert!(reduced_rent < 550);
}
