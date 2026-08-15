use super::*;
use crate::model::{
    Condition, DealArchetype, OwnedProperty, Player, Property, ResearchLevel, WalkawayStyle,
};
use std::collections::HashMap;

fn market(budget_modifier: f32) -> MarketEvent {
    MarketEvent {
        title: "Test".to_string(),
        items: vec![],
        suburb_modifiers: HashMap::new(),
        renovator_modifier: 0.0,
        buyer_budget_modifier: budget_modifier,
        strategy_effect: "Test strategy effect".to_string(),
    }
}

#[test]
fn debt_interest_rounds_up_to_hundreds() {
    let steady = market(0.0);
    assert_eq!(weekly_debt_interest(400_000, &steady), 400);
    assert_eq!(weekly_debt_interest(410_000, &steady), 400);
    assert!(weekly_debt_interest(400_000, &market(-0.04)) > weekly_debt_interest(400_000, &steady));
}

#[test]
fn campaign_ends_after_goal_or_deadline() {
    let steady = market(0.0);
    let mut player = Player::new();
    assert_eq!(
        campaign_status(&player, &steady, 41),
        CampaignStatus::Failed
    );
    assert_eq!(
        campaign_status(&player, &steady, 40),
        CampaignStatus::Active
    );

    for id in 0..3 {
        let mut owned = owned_property(id);
        owned.is_leased = true;
        owned.weekly_rent = 550;
        player.properties.push(owned);
    }
    player.cash = 300_000;
    assert_eq!(campaign_status(&player, &steady, 12), CampaignStatus::Won);
}

fn owned_property(id: usize) -> OwnedProperty {
    let property = Property {
        id,
        address: format!("{id} Test Street"),
        suburb: "Westport".to_string(),
        bedrooms: 3,
        bathrooms: 1,
        condition: Condition::Solid,
        land_size: 450,
        market_value: 500_000,
        guide_price: 450_000,
        reserve_price: 470_000,
        appeal: 55,
        renovation_potential: 50,
        hidden_defect_risk: 0.1,
        holding_cost_per_week: 120,
        buyer_demand: 60,
        deal_archetype: DealArchetype::RentalHold,
        thesis: String::new(),
        main_risk: String::new(),
        best_strategy: String::new(),
        bad_strategy: String::new(),
        notes: String::new(),
    };
    OwnedProperty::new(
        property,
        470_000,
        18_000,
        56_000,
        414_000,
        480_000,
        ResearchLevel::StreetScan,
        WalkawayStyle::Balanced,
    )
}

#[test]
fn starter_player_has_no_weekly_pressure() {
    let player = Player::new();
    assert_eq!(weekly_debt_interest(player.debt, &market(0.0)), 0);
    assert_eq!(weekly_holding_cost(&player), 0);
}
