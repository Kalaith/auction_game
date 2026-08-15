use super::*;

#[test]
fn a_new_investor_starts_with_an_empty_career_record() {
    let player = Player::new();

    assert_eq!(player.career.auctions_attended, 0);
    assert_eq!(player.career.homes_bought, 0);
    assert_eq!(player.career.realized_profit, 0);
    assert_eq!(player.career.unused_registrations, 0);
    assert_eq!(player.career.rent_reviews_completed, 0);
    assert_eq!(player.career.review_vacancies, 0);
    assert!(player.rival_notebook.is_empty());
}

#[test]
fn older_career_ledgers_gain_empty_tenancy_outcomes() {
    let career: CareerRecord = serde_json::from_str(r#"{"auctions_attended": 4}"#)
        .expect("older career record should remain readable");

    assert_eq!(career.auctions_attended, 4);
    assert_eq!(career.rent_reviews_completed, 0);
    assert_eq!(career.review_vacancies, 0);
}

#[test]
fn older_saves_gain_an_empty_career_record() {
    let player: Player = serde_json::from_str(
        r#"{
            "cash": 184000,
            "debt": 0,
            "properties": [],
            "reputation": 2
        }"#,
    )
    .expect("old player save should remain readable");

    assert_eq!(player.cash, 184_000);
    assert_eq!(player.reputation, 2);
    assert_eq!(player.career.auctions_attended, 0);
    assert_eq!(player.career.unused_registrations, 0);
    assert!(player.rival_notebook.is_empty());
}

#[test]
fn weekly_restraint_is_capped_to_the_two_registration_allowance() {
    let mut career = CareerRecord::default();

    career.record_unused_registrations(1);
    career.record_unused_registrations(8);

    assert_eq!(career.unused_registrations, 3);
}

#[test]
fn an_older_owned_home_gains_idle_leasing_and_review_state() {
    let data = crate::data::GameData::load();
    let property = crate::model::Property::from_template(&data.properties[0]);
    let owned = OwnedProperty::new(
        property,
        500_000,
        20_000,
        50_000,
        450_000,
        520_000,
        ResearchLevel::StreetScan,
        WalkawayStyle::Balanced,
    );
    let mut json = serde_json::to_value(owned).unwrap();
    let object = json.as_object_mut().unwrap();
    object.remove("leasing_weeks_remaining");
    object.remove("next_rent_review_week");

    let restored: OwnedProperty = serde_json::from_value(json).expect("legacy home should load");

    assert_eq!(restored.leasing_weeks_remaining, 0);
    assert_eq!(restored.next_rent_review_week, 0);
}
