use super::*;
use crate::model::{CompletedUpgrade, ContractorTier, OwnedProperty, Property, PropertyTemplate};
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

#[test]
fn completed_improvements_raise_the_next_rent_appraisal() {
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
    let base_rent = weekly_rent_for_owned(&owned, &market());
    owned.upgrades.push(CompletedUpgrade {
        upgrade_id: "kitchen_refresh".to_string(),
        name: "Kitchen Refresh".to_string(),
        contractor: ContractorTier::Reliable,
        actual_cost: 26_000,
        value_boost: 41_000,
        appeal_boost: 12,
        sale_emotion_boost: 7,
        removes_defect: false,
        weeks_taken: 2,
        note: String::new(),
    });

    assert!(weekly_rent_for_owned(&owned, &market()) > base_rent);
}

#[test]
fn ending_a_tenancy_returns_the_home_to_vacant_state() {
    let property = property();
    let mut owned = OwnedProperty::new(
        property,
        470_000,
        18_000,
        56_000,
        414_000,
        480_000,
        crate::model::ResearchLevel::StreetScan,
        crate::model::WalkawayStyle::Balanced,
    );
    owned.is_leased = true;
    owned.weekly_rent = 530;

    let cost = end_tenancy(&mut owned);

    assert_eq!(cost, 530);
    assert!(!owned.is_leased);
    assert_eq!(owned.weekly_rent, 0);
}

#[test]
fn advertising_takes_a_week_before_rent_can_be_collected() {
    let mut owned = OwnedProperty::new(
        property(),
        470_000,
        18_000,
        56_000,
        414_000,
        480_000,
        crate::model::ResearchLevel::StreetScan,
        crate::model::WalkawayStyle::Balanced,
    );
    assert!(start_leasing_campaign(&mut owned, 530));
    assert_eq!(effective_weekly_rent(&owned), 0);
    let mut player = Player::new();
    player.properties.push(owned);

    assert_eq!(portfolio_rental_snapshot(&player).gross_rent, 0);
    let notices = progress_leasing_campaigns(&mut player);

    assert_eq!(notices.len(), 1);
    assert!(player.properties[0].is_leased);
    assert_eq!(portfolio_rental_snapshot(&player).gross_rent, 530);
}

#[test]
fn a_completed_tenancy_schedules_a_visible_rent_review() {
    let mut owned = OwnedProperty::new(
        property(),
        470_000,
        18_000,
        47_000,
        423_000,
        480_000,
        crate::model::ResearchLevel::StreetScan,
        crate::model::WalkawayStyle::Balanced,
    );
    owned.weeks_held = 3;
    start_leasing_campaign(&mut owned, 530);
    let mut player = Player::new();
    player.properties.push(owned);

    progress_leasing_campaigns(&mut player);

    assert_eq!(player.properties[0].next_rent_review_week, 11);
    player.properties[0].weeks_held = 11;
    assert!(rent_review_due(&player.properties[0]));
}

#[test]
fn safe_review_renews_while_an_ambitious_low_demand_ask_can_create_vacancy() {
    let mut safe = OwnedProperty::new(
        property(),
        470_000,
        18_000,
        47_000,
        423_000,
        480_000,
        crate::model::ResearchLevel::StreetScan,
        crate::model::WalkawayStyle::Balanced,
    );
    safe.is_leased = true;
    safe.weekly_rent = 530;
    safe.weeks_held = 10;
    safe.next_rent_review_week = 10;
    let mut ambitious = safe.clone();
    ambitious.property.buyer_demand = 40;
    ambitious.weekly_rent = 600;

    assert_eq!(
        rent_review_outlook(&ambitious, &market()),
        RentReviewOutlook::VacancyRisk
    );

    assert_eq!(
        resolve_rent_review(&mut safe, &market(), false),
        Some(RentReviewOutcome::Renewed(530))
    );
    assert!(!rent_review_due(&safe));
    assert_eq!(
        resolve_rent_review(&mut ambitious, &market(), true),
        Some(RentReviewOutcome::Vacated(620))
    );
    assert!(!ambitious.is_leased);
}

#[test]
fn a_home_already_on_the_rental_market_cannot_be_listed_twice() {
    let mut owned = OwnedProperty::new(
        property(),
        470_000,
        18_000,
        56_000,
        414_000,
        480_000,
        crate::model::ResearchLevel::StreetScan,
        crate::model::WalkawayStyle::Balanced,
    );

    assert!(start_leasing_campaign(&mut owned, 530));
    assert!(!start_leasing_campaign(&mut owned, 560));
    assert_eq!(owned.weekly_rent, 530);
}
