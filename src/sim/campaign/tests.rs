use super::*;
use crate::data::GameData;
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
        campaign_status(&player, &steady, 25),
        CampaignStatus::Failed
    );
    assert_eq!(
        campaign_status(&player, &steady, 24),
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

#[test]
fn rent_offsets_interest_holding_and_management_before_cash_moves() {
    let steady = market(0.0);
    let mut player = Player::new();
    let mut owned = owned_property(1);
    owned.is_leased = true;
    owned.weekly_rent = 550;
    player.debt = owned.debt;
    player.properties.push(owned);
    let starting_cash = player.cash;

    let pressure = apply_weekly_pressure(&mut player, &steady);
    let expected_cash = starting_cash + pressure.rental_income - pressure.total;

    assert_eq!(player.cash, expected_cash);
    assert!(pressure.rental_operating_cost > 0);
}

#[test]
fn a_disciplined_three_rental_path_can_complete_the_real_campaign() {
    let data = GameData::load();
    let market = &data.market_events[0];
    let mut player = Player::new();

    for id in [6, 8] {
        acquire_authored_rental(&data, &mut player, id, market);
    }
    player.reputation = 3;
    acquire_authored_rental(&data, &mut player, 5, market);

    assert!(
        crate::sim::rental::portfolio_rental_snapshot(&player).gross_rent
            >= CAMPAIGN_GOAL_WEEKLY_RENT
    );
    let worth = crate::sim::valuation::net_worth(&player, market);
    assert!(
        worth >= CAMPAIGN_GOAL_NET_WORTH,
        "three-rental path finished at {worth} net worth"
    );
    assert_eq!(campaign_status(&player, market, 12), CampaignStatus::Won);
}

#[test]
fn postmortem_names_the_largest_normalized_campaign_gap() {
    let market = market(0.0);
    let mut player = Player::new();
    player.cash = 230_000;

    let assessment = assess_campaign(&player, &market);

    assert_eq!(assessment.homes_short, 3);
    assert_eq!(assessment.rent_short, CAMPAIGN_GOAL_WEEKLY_RENT);
    assert_eq!(assessment.net_worth_short, 10_000);
    assert!(matches!(
        assessment.priority,
        CampaignPriority::Rent | CampaignPriority::Homes
    ));
}

#[test]
fn completed_brief_has_no_postmortem_shortfalls() {
    let market = market(0.0);
    let mut player = Player::new();
    for id in 0..3 {
        let mut owned = owned_property(id);
        owned.is_leased = true;
        owned.weekly_rent = 500;
        player.properties.push(owned);
    }
    player.cash = 300_000;

    let assessment = assess_campaign(&player, &market);

    assert_eq!(assessment.homes_short, 0);
    assert_eq!(assessment.rent_short, 0);
    assert_eq!(assessment.net_worth_short, 0);
    assert_eq!(assessment.priority, CampaignPriority::Complete);
}

fn acquire_authored_rental(
    data: &GameData,
    player: &mut Player,
    property_id: usize,
    market: &MarketEvent,
) {
    let property = Property::from_template(
        data.properties
            .iter()
            .find(|property| property.id == property_id)
            .expect("authored campaign property should exist"),
    );
    let price = property.reserve_price - 10_000;
    assert!(crate::sim::finance::finance_snapshot(player, market, price).can_buy);
    let property_deposit = crate::sim::valuation::deposit(price);
    let property_debt = price - property_deposit;
    player.cash -= crate::sim::valuation::cash_needed_to_settle(price);
    player.debt += property_debt;
    let mut owned = OwnedProperty::new(
        property,
        price,
        crate::sim::valuation::purchase_fees(price),
        property_deposit,
        property_debt,
        price,
        ResearchLevel::BuildingInspection,
        WalkawayStyle::Balanced,
    );
    owned.is_leased = true;
    owned.weekly_rent = crate::sim::rental::weekly_rent_for_owned(&owned, market);
    player.cash -= crate::sim::rental::leasing_cost(owned.weekly_rent);
    player.properties.push(owned);
}
