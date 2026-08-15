use super::*;
use crate::data::GameData;
use crate::model::{MarketEvent, OwnedProperty, Property, ResearchLevel, WalkawayStyle};
use crate::sim::rental::weekly_rent_for_owned;
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
fn rate_changes_move_finance_limit() {
    let player = Player::new();
    assert!(borrowing_limit(&player, &market(0.03)) > borrowing_limit(&player, &market(-0.03)));
}

#[test]
fn finance_snapshot_blocks_deals_above_limit() {
    let player = Player::new();
    let snapshot = finance_snapshot(&player, &market(0.0), 1_200_000);
    assert!(!snapshot.can_buy);
    assert_eq!(snapshot.stress, FinanceStress::Maxed);
}

#[test]
fn two_performing_rentals_and_discipline_open_the_third_purchase() {
    let data = GameData::load();
    let market = &data.market_events[0];
    let mut player = Player::new();

    for id in [6, 8] {
        let template = data
            .properties
            .iter()
            .find(|property| property.id == id)
            .unwrap();
        acquire_and_lease(&mut player, Property::from_template(template), market);
    }
    player.reputation = 1;
    let third = data
        .properties
        .iter()
        .find(|property| property.id == 3)
        .unwrap();

    let finance = finance_snapshot(&player, market, third.reserve_price);

    assert!(finance.can_buy);
    assert!(
        player
            .properties
            .iter()
            .map(|owned| owned.weekly_rent)
            .sum::<i64>()
            >= 950
    );
}

fn acquire_and_lease(player: &mut Player, property: Property, market: &MarketEvent) {
    let price = property.reserve_price;
    assert!(finance_snapshot(player, market, price).can_buy);
    let property_deposit = deposit(price);
    let property_debt = price - property_deposit;
    player.cash -= cash_needed_to_settle(price);
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
    owned.weekly_rent = weekly_rent_for_owned(&owned, market);
    player.properties.push(owned);
}
