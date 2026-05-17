use crate::model::{MarketEvent, Player};
use crate::sim::valuation::{cash_needed_to_settle, deposit, net_worth, round_down_to_increment};

const STARTER_BORROWING_LIMIT: i64 = 620_000;
const REPUTATION_BONUS: i64 = 35_000;
const NET_WORTH_LEVERAGE: f32 = 0.85;
const BID_INCREMENT: i64 = 10_000;
const MAX_TEST_PRICE: i64 = 2_500_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinanceStress {
    Healthy,
    Tight,
    Maxed,
}

impl FinanceStress {
    pub fn label(self) -> &'static str {
        match self {
            FinanceStress::Healthy => "Finance OK",
            FinanceStress::Tight => "Tight Finance",
            FinanceStress::Maxed => "Bank Says No",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FinanceSnapshot {
    pub headroom_after: i64,
    pub cash_after_settle: i64,
    pub can_buy: bool,
    pub stress: FinanceStress,
}

pub fn borrowing_limit(player: &Player, market: &MarketEvent) -> i64 {
    let base = STARTER_BORROWING_LIMIT
        + i64::from(player.reputation.max(0)) * REPUTATION_BONUS
        + (net_worth(player, market).max(0) as f32 * NET_WORTH_LEVERAGE) as i64;
    round_down_to_increment(
        (base as f32 * (1.0 + market.buyer_budget_modifier)).max(0.0) as i64,
        BID_INCREMENT,
    )
}

pub fn finance_snapshot(player: &Player, market: &MarketEvent, price: i64) -> FinanceSnapshot {
    let limit = borrowing_limit(player, market);
    let debt_needed = price - deposit(price);
    let debt_after = player.debt + debt_needed;
    let headroom_after = limit - debt_after;
    let cash_after_settle = player.cash - cash_needed_to_settle(price);
    let can_buy = cash_after_settle >= 0 && headroom_after >= 0;
    let stress = if !can_buy || headroom_after < 20_000 {
        FinanceStress::Maxed
    } else if cash_after_settle < 18_000 || headroom_after < 80_000 {
        FinanceStress::Tight
    } else {
        FinanceStress::Healthy
    };

    FinanceSnapshot {
        headroom_after,
        cash_after_settle,
        can_buy,
        stress,
    }
}

pub fn max_financeable_bid(player: &Player, market: &MarketEvent) -> i64 {
    let mut best = 0;
    let mut price = BID_INCREMENT;
    while price <= MAX_TEST_PRICE {
        if finance_snapshot(player, market, price).can_buy {
            best = price;
            price += BID_INCREMENT;
        } else {
            break;
        }
    }
    best
}

#[cfg(test)]
mod tests {
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
}
