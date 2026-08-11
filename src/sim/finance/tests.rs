use super::*;
use crate::model::MarketEvent;
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
