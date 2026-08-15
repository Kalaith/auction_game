use crate::model::{
    CampaignStatus, MarketEvent, Player, CAMPAIGN_GOAL_NET_WORTH, CAMPAIGN_GOAL_PROPERTIES,
    CAMPAIGN_GOAL_WEEKLY_RENT, CAMPAIGN_MAX_WEEKS,
};
use crate::sim::rental::{apply_rental_income, portfolio_rental_snapshot};
use crate::sim::valuation::net_worth;
use serde::{Deserialize, Serialize};

const WEEKLY_DEBT_INTEREST_RATE: f32 = 0.00095;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WeeklyPressure {
    pub rental_income: i64,
    pub rental_operating_cost: i64,
    pub debt_interest: i64,
    pub holding_cost: i64,
    pub total: i64,
    pub shortfall_added_to_debt: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignAssessment {
    pub homes_short: usize,
    pub rent_short: i64,
    pub net_worth_short: i64,
    pub priority: CampaignPriority,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CampaignPriority {
    Complete,
    Homes,
    Rent,
    NetWorth,
}

impl CampaignAssessment {
    pub fn priority_advice(&self) -> &'static str {
        match self.priority {
            CampaignPriority::Complete => {
                "All three constraints work together. The portfolio is ready for its next cycle."
            }
            CampaignPriority::Homes => {
                "Door count was the binding gap. Preserve registrations and deposits for financeable homes instead of chasing every room."
            }
            CampaignPriority::Rent => {
                "Rent was the binding gap. Compare weekly rent against settlement cash, repair needs, and debt before raising the paddle."
            }
            CampaignPriority::NetWorth => {
                "Equity was the binding gap. Buy below value, create measured upside, and stop bidding when auction pressure erases the margin."
            }
        }
    }
}

pub fn apply_weekly_pressure(player: &mut Player, market: &MarketEvent) -> WeeklyPressure {
    let rental = apply_rental_income(player);
    let debt_interest = weekly_debt_interest(player.debt, market);
    let holding_cost = weekly_holding_cost(player);
    let total = debt_interest + holding_cost + rental.operating_cost;
    let net_cost = total - rental.gross_rent;
    let shortfall_added_to_debt = if net_cost <= 0 {
        player.cash += -net_cost;
        0
    } else if player.cash >= net_cost {
        player.cash -= net_cost;
        0
    } else {
        let shortfall = net_cost - player.cash;
        player.cash = 0;
        player.debt += shortfall;
        shortfall
    };

    for property in &mut player.properties {
        property.weeks_held += 1;
    }

    WeeklyPressure {
        rental_income: rental.gross_rent,
        rental_operating_cost: rental.operating_cost,
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

    let rate = weekly_interest_rate(market);
    (debt as f32 * rate).ceil() as i64
}

pub fn annual_interest_rate_percent(market: &MarketEvent) -> f32 {
    weekly_interest_rate(market) * 52.0 * 100.0
}

fn weekly_interest_rate(market: &MarketEvent) -> f32 {
    (WEEKLY_DEBT_INTEREST_RATE - market.buyer_budget_modifier * 0.012).max(0.0005)
}

pub fn weekly_holding_cost(player: &Player) -> i64 {
    player
        .properties
        .iter()
        .map(|owned| owned.property.holding_cost_per_week)
        .sum()
}

pub fn portfolio_weekly_cashflow(player: &Player, market: &MarketEvent) -> i64 {
    let rental = portfolio_rental_snapshot(player);
    rental.gross_rent
        - rental.operating_cost
        - weekly_debt_interest(player.debt, market)
        - weekly_holding_cost(player)
}

pub fn campaign_status(player: &Player, market: &MarketEvent, week: u32) -> CampaignStatus {
    let rent = portfolio_rental_snapshot(player).gross_rent;
    if player.properties.len() >= CAMPAIGN_GOAL_PROPERTIES
        && rent >= CAMPAIGN_GOAL_WEEKLY_RENT
        && net_worth(player, market) >= CAMPAIGN_GOAL_NET_WORTH
    {
        CampaignStatus::Won
    } else if week > CAMPAIGN_MAX_WEEKS {
        CampaignStatus::Failed
    } else {
        CampaignStatus::Active
    }
}

pub fn campaign_progress(player: &Player, market: &MarketEvent) -> (usize, i64, i64) {
    (
        player.properties.len(),
        portfolio_rental_snapshot(player).gross_rent,
        net_worth(player, market),
    )
}

pub fn assess_campaign(player: &Player, market: &MarketEvent) -> CampaignAssessment {
    let (homes, rent, worth) = campaign_progress(player, market);
    let homes_short = CAMPAIGN_GOAL_PROPERTIES.saturating_sub(homes);
    let rent_short = (CAMPAIGN_GOAL_WEEKLY_RENT - rent).max(0);
    let net_worth_short = (CAMPAIGN_GOAL_NET_WORTH - worth).max(0);
    let normalized = [
        (
            homes_short as f32 / CAMPAIGN_GOAL_PROPERTIES as f32,
            CampaignPriority::Homes,
        ),
        (
            rent_short as f32 / CAMPAIGN_GOAL_WEEKLY_RENT as f32,
            CampaignPriority::Rent,
        ),
        (
            net_worth_short as f32 / CAMPAIGN_GOAL_NET_WORTH as f32,
            CampaignPriority::NetWorth,
        ),
    ];
    let priority = if homes_short == 0 && rent_short == 0 && net_worth_short == 0 {
        CampaignPriority::Complete
    } else {
        normalized
            .into_iter()
            .max_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, priority)| priority)
            .unwrap_or(CampaignPriority::Complete)
    };

    CampaignAssessment {
        homes_short,
        rent_short,
        net_worth_short,
        priority,
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

#[cfg(test)]
mod tests;
