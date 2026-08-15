use super::*;
use crate::model::{
    Condition, DealArchetype, MaintenanceIssue, MaintenanceKind, OwnedProperty, Property,
    ResearchLevel, WalkawayStyle,
};
use std::collections::HashMap;

#[test]
fn purchase_cash_needed_includes_deposit_and_fees() {
    assert_eq!(deposit(600_000), 72_000);
    assert_eq!(purchase_fees(600_000), 23_000);
    assert_eq!(cash_needed_to_settle(600_000), 95_000);
}

#[test]
fn unresolved_maintenance_reduces_the_sale_value_more_than_the_repair_cost() {
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
    let market = MarketEvent {
        title: String::new(),
        items: Vec::new(),
        suburb_modifiers: HashMap::new(),
        renovator_modifier: 0.0,
        buyer_budget_modifier: 0.0,
        strategy_effect: String::new(),
    };
    let maintained_value = current_value(&owned, &market);
    owned.maintenance_issue = Some(MaintenanceIssue {
        kind: MaintenanceKind::RoofRepair,
        repair_cost: 8_000,
        weekly_rent_loss: 160,
        description: String::new(),
    });

    assert!(maintained_value - current_value(&owned, &market) > 8_000);
}
