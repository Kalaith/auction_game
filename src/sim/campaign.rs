use crate::model::{
    CampaignStatus, MarketEvent, Player, CAMPAIGN_GOAL_NET_WORTH, CAMPAIGN_MAX_WEEKS,
};

const WEEKLY_DEBT_INTEREST_RATE: f32 = 0.0015;

#[derive(Clone, Debug)]
pub struct WeeklyPressure {
    pub debt_interest: i64,
    pub holding_cost: i64,
    pub total: i64,
    pub shortfall_added_to_debt: i64,
}

pub fn apply_weekly_pressure(player: &mut Player, market: &MarketEvent) -> WeeklyPressure {
    let debt_interest = weekly_debt_interest(player.debt, market);
    let holding_cost = weekly_holding_cost(player);
    let total = debt_interest + holding_cost;
    let shortfall_added_to_debt = if player.cash >= total {
        player.cash -= total;
        0
    } else {
        let shortfall = total - player.cash;
        player.cash = 0;
        player.debt += shortfall;
        shortfall
    };

    for property in &mut player.properties {
        property.weeks_held += 1;
    }

    WeeklyPressure {
        debt_interest,
        holding_cost,
        total,
        shortfall_added_to_debt,
    }
}

pub fn weekly_debt_interest(debt: i64, market: &MarketEvent) -> i64 {
    if debt <= 0 {
        return 0;
    }

    let rate = (WEEKLY_DEBT_INTEREST_RATE - market.buyer_budget_modifier * 0.012).max(0.0005);
    round_up_to_100(debt as f32 * rate)
}

pub fn weekly_holding_cost(player: &Player) -> i64 {
    player
        .properties
        .iter()
        .map(|owned| owned.property.holding_cost_per_week)
        .sum()
}

pub fn campaign_status(week: u32, net_worth: i64) -> CampaignStatus {
    if net_worth >= CAMPAIGN_GOAL_NET_WORTH {
        CampaignStatus::Won
    } else if week > CAMPAIGN_MAX_WEEKS {
        CampaignStatus::Failed
    } else {
        CampaignStatus::Active
    }
}

pub fn suburb_is_unlocked(suburb: &str, week: u32, net_worth: i64, reputation: i32) -> bool {
    match suburb {
        "Southmere" | "Ridgefield" | "Westport" => true,
        "Northbank" => week >= 4 || net_worth >= 250_000 || reputation >= 1,
        "Eastvale" => week >= 8 || net_worth >= 500_000 || reputation >= 2,
        _ => true,
    }
}

pub fn next_unlock_note(week: u32, net_worth: i64, reputation: i32) -> &'static str {
    if !suburb_is_unlocked("Northbank", week, net_worth, reputation) {
        "Next unlock: Northbank at week 4, $250k net worth, or +1 reputation."
    } else if !suburb_is_unlocked("Eastvale", week, net_worth, reputation) {
        "Next unlock: Eastvale at week 8, $500k net worth, or +2 reputation."
    } else {
        "All starter suburbs are unlocked."
    }
}

fn round_up_to_100(value: f32) -> i64 {
    ((value / 100.0).ceil() as i64) * 100
}

#[cfg(test)]
mod tests {
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
        assert!(
            weekly_debt_interest(400_000, &market(-0.04)) > weekly_debt_interest(400_000, &steady)
        );
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
}
