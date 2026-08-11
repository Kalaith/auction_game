use super::*;
use crate::model::Player;
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
    assert_eq!(weekly_debt_interest(400_000, &steady), 600);
    assert_eq!(weekly_debt_interest(410_000, &steady), 700);
    assert!(weekly_debt_interest(400_000, &market(-0.04)) > weekly_debt_interest(400_000, &steady));
}

#[test]
fn campaign_ends_after_goal_or_deadline() {
    assert_eq!(campaign_status(12, 1_000_000), CampaignStatus::Won);
    assert_eq!(campaign_status(53, 900_000), CampaignStatus::Failed);
    assert_eq!(campaign_status(52, 900_000), CampaignStatus::Active);
}

#[test]
fn starter_player_has_no_weekly_pressure() {
    let player = Player::new();
    assert_eq!(weekly_debt_interest(player.debt, &market(0.0)), 0);
    assert_eq!(weekly_holding_cost(&player), 0);
}
