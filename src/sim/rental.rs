use crate::model::{Condition, DealArchetype, MarketEvent, OwnedProperty, Player, Property};
use crate::sim::maintenance::effective_weekly_rent;

const RENT_REVIEW_TERM_WEEKS: u32 = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RentalSnapshot {
    pub gross_rent: i64,
    pub operating_cost: i64,
    pub net_income: i64,
}

pub fn weekly_rent_for(property: &Property, market: &MarketEvent) -> i64 {
    let base_yield = match property.deal_archetype {
        DealArchetype::RentalHold | DealArchetype::QuietBargain => 0.057,
        DealArchetype::RiskyFixer | DealArchetype::LandValuePlay => 0.052,
        DealArchetype::PrettyTrap | DealArchetype::HotSuburbFomo => 0.044,
        DealArchetype::RenovatorBait | DealArchetype::AuctionTrap => 0.049,
    };
    let condition_adjustment = match property.condition {
        Condition::Rough => -0.08,
        Condition::Tired => -0.03,
        Condition::Solid => 0.03,
        Condition::Premium => 0.07,
    };
    let demand_adjustment = (property.buyer_demand - 55) as f32 / 500.0;
    let market_adjustment = market.suburb_modifier(&property.suburb) * 0.25;
    round_to_ten(
        property.market_value as f32
            * base_yield
            * (1.0 + condition_adjustment + demand_adjustment + market_adjustment)
            / 52.0,
    )
}

pub fn weekly_rent_for_owned(owned: &OwnedProperty, market: &MarketEvent) -> i64 {
    let improvement_rent: i64 = owned
        .upgrades
        .iter()
        .filter(|upgrade| upgrade.upgrade_id != "staging")
        .map(|upgrade| round_to_ten(upgrade.value_boost as f32 * 0.035 / 52.0))
        .sum();
    weekly_rent_for(&owned.property, market) + improvement_rent
}

pub fn leasing_cost(weekly_rent: i64) -> i64 {
    weekly_rent * 2
}

pub fn start_leasing_campaign(owned: &mut OwnedProperty, weekly_rent: i64) -> bool {
    if owned.is_leased || owned.leasing_weeks_remaining > 0 || weekly_rent <= 0 {
        return false;
    }
    owned.weekly_rent = weekly_rent;
    owned.leasing_weeks_remaining = 1;
    true
}

pub fn progress_leasing_campaigns(player: &mut Player) -> Vec<String> {
    let mut notices = Vec::new();
    for owned in &mut player.properties {
        if owned.is_leased || owned.leasing_weeks_remaining == 0 {
            continue;
        }
        owned.leasing_weeks_remaining -= 1;
        if owned.leasing_weeks_remaining == 0 {
            owned.is_leased = true;
            owned.next_rent_review_week = owned.weeks_held + RENT_REVIEW_TERM_WEEKS;
            notices.push(format!(
                "{} leased at {} per week.",
                owned.property.address,
                crate::ui::format_money(owned.weekly_rent)
            ));
        }
    }
    notices
}

pub fn end_tenancy(owned: &mut OwnedProperty) -> i64 {
    if !owned.is_leased {
        return 0;
    }
    let turnover_cost = owned.weekly_rent;
    owned.is_leased = false;
    owned.weekly_rent = 0;
    owned.leasing_weeks_remaining = 0;
    owned.next_rent_review_week = 0;
    turnover_cost
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RentReviewOutcome {
    Renewed(i64),
    Raised(i64),
    Vacated(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RentReviewOutlook {
    AppraisalSupported,
    DemandSupported,
    VacancyRisk,
}

impl RentReviewOutlook {
    pub fn label(self) -> &'static str {
        match self {
            RentReviewOutlook::AppraisalSupported => "At appraisal · low vacancy risk",
            RentReviewOutlook::DemandSupported => "Strong demand · likely to hold",
            RentReviewOutlook::VacancyRisk => "Above appraisal · vacancy risk",
        }
    }
}

pub fn rent_review_due(owned: &OwnedProperty) -> bool {
    owned.is_leased
        && owned.next_rent_review_week > 0
        && owned.weeks_held >= owned.next_rent_review_week
}

pub fn proposed_review_rent(owned: &OwnedProperty, market: &MarketEvent) -> i64 {
    weekly_rent_for_owned(owned, market).max(owned.weekly_rent + 20)
}

pub fn rent_review_outlook(owned: &OwnedProperty, market: &MarketEvent) -> RentReviewOutlook {
    let proposed = proposed_review_rent(owned, market);
    let market_rent = weekly_rent_for_owned(owned, market);
    let demand_score = owned.property.buyer_demand
        + (market.suburb_modifier(&owned.property.suburb) * 100.0) as i32;
    if proposed <= market_rent {
        RentReviewOutlook::AppraisalSupported
    } else if demand_score >= 65 {
        RentReviewOutlook::DemandSupported
    } else {
        RentReviewOutlook::VacancyRisk
    }
}

pub fn resolve_rent_review(
    owned: &mut OwnedProperty,
    market: &MarketEvent,
    test_market: bool,
) -> Option<RentReviewOutcome> {
    if !rent_review_due(owned) {
        return None;
    }
    if !test_market {
        owned.next_rent_review_week = owned.weeks_held + RENT_REVIEW_TERM_WEEKS;
        return Some(RentReviewOutcome::Renewed(owned.weekly_rent));
    }

    let proposed = proposed_review_rent(owned, market);
    if rent_review_outlook(owned, market) != RentReviewOutlook::VacancyRisk {
        owned.weekly_rent = proposed;
        owned.next_rent_review_week = owned.weeks_held + RENT_REVIEW_TERM_WEEKS;
        Some(RentReviewOutcome::Raised(proposed))
    } else {
        owned.is_leased = false;
        owned.weekly_rent = 0;
        owned.next_rent_review_week = 0;
        Some(RentReviewOutcome::Vacated(proposed))
    }
}

pub fn portfolio_rental_snapshot(player: &Player) -> RentalSnapshot {
    player
        .properties
        .iter()
        .filter(|owned| owned.is_leased)
        .fold(RentalSnapshot::default(), |mut total, owned| {
            let collected_rent = effective_weekly_rent(owned);
            let operating_cost = rental_management_cost(collected_rent);
            total.gross_rent += collected_rent;
            total.operating_cost += operating_cost;
            total.net_income += owned.weekly_rent - operating_cost;
            total
        })
}

pub fn apply_rental_income(player: &mut Player) -> RentalSnapshot {
    let snapshot = portfolio_rental_snapshot(player);
    for owned in &mut player.properties {
        if owned.is_leased {
            let collected_rent = effective_weekly_rent(owned);
            let operating_cost = rental_management_cost(collected_rent);
            owned.rent_received += collected_rent;
            owned.operating_spend += operating_cost;
        }
    }
    snapshot
}

pub fn rental_management_cost(weekly_rent: i64) -> i64 {
    round_to_ten(weekly_rent as f32 * 0.12)
}

fn round_to_ten(value: f32) -> i64 {
    ((value / 10.0).round() as i64) * 10
}

#[cfg(test)]
mod tests;
