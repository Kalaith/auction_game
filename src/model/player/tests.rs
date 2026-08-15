use super::*;

#[test]
fn a_new_investor_starts_with_an_empty_career_record() {
    let player = Player::new();

    assert_eq!(player.career.auctions_attended, 0);
    assert_eq!(player.career.homes_bought, 0);
    assert_eq!(player.career.realized_profit, 0);
    assert!(player.rival_notebook.is_empty());
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
    assert!(player.rival_notebook.is_empty());
}
