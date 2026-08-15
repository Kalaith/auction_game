use crate::model::{MarketEvent, Player, Property, PropertyId};
use crate::sim::campaign::weekly_debt_interest;
use crate::sim::rental::{portfolio_rental_snapshot, rental_management_cost, weekly_rent_for};
use crate::sim::valuation::{
    cash_needed_to_settle, current_value, deposit, net_worth, round_down_to_increment,
};

const STARTER_BORROWING_LIMIT: i64 = 620_000;
const REPUTATION_BONUS: i64 = 35_000;
const NET_WORTH_LEVERAGE: f32 = 0.85;
const RENTAL_INCOME_LEVERAGE: i64 = 280;
const BID_INCREMENT: i64 = 10_000;
const MAX_TEST_PRICE: i64 = 2_500_000;
const REFINANCE_LVR: f32 = 0.80;
pub const REFINANCE_FEE: i64 = 2_000;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RentalUnderwrite {
    pub gross_rent: i64,
    pub management: i64,
    pub property_cost: i64,
    pub loan_interest: i64,
    pub net_cashflow: i64,
}

pub fn rental_underwrite(
    property: &Property,
    market: &MarketEvent,
    purchase_price: i64,
) -> RentalUnderwrite {
    let gross_rent = weekly_rent_for(property, market);
    let management = rental_management_cost(gross_rent);
    let property_cost = property.holding_cost_per_week;
    let loan_interest = weekly_debt_interest(purchase_price - deposit(purchase_price), market);
    RentalUnderwrite {
        gross_rent,
        management,
        property_cost,
        loan_interest,
        net_cashflow: gross_rent - management - property_cost - loan_interest,
    }
}

pub fn borrowing_limit(player: &Player, market: &MarketEvent) -> i64 {
    let base = STARTER_BORROWING_LIMIT
        + i64::from(player.reputation.max(0)) * REPUTATION_BONUS
        + (net_worth(player, market).max(0) as f32 * NET_WORTH_LEVERAGE) as i64
        + portfolio_rental_snapshot(player).gross_rent * RENTAL_INCOME_LEVERAGE;
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

pub fn pay_down_principal(player: &mut Player, property_id: PropertyId, requested: i64) -> i64 {
    let Some(index) = player
        .properties
        .iter()
        .position(|owned| owned.property.id == property_id)
    else {
        return 0;
    };
    let payment = requested.min(player.properties[index].debt).max(0);
    if payment == 0 || player.cash < payment {
        return 0;
    }

    player.cash -= payment;
    player.debt -= payment;
    player.properties[index].debt -= payment;
    payment
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefinanceResult {
    pub debt_added: i64,
    pub fee: i64,
    pub cash_released: i64,
}

pub fn refinance_capacity(player: &Player, property_id: PropertyId, market: &MarketEvent) -> i64 {
    let Some(owned) = player
        .properties
        .iter()
        .find(|owned| owned.property.id == property_id)
    else {
        return 0;
    };
    if !owned.is_leased
        || owned.weeks_held < 4
        || owned.active_renovation.is_some()
        || owned.maintenance_issue.is_some()
    {
        return 0;
    }

    let property_limit = round_down_to_increment(
        (current_value(owned, market) as f32 * REFINANCE_LVR) as i64,
        BID_INCREMENT,
    );
    let property_room = (property_limit - owned.debt).max(0);
    let bank_room = (borrowing_limit(player, market) - player.debt - BID_INCREMENT).max(0);
    round_down_to_increment(property_room.min(bank_room), BID_INCREMENT)
}

pub fn refinance_property(
    player: &mut Player,
    property_id: PropertyId,
    market: &MarketEvent,
) -> Option<RefinanceResult> {
    let debt_added = refinance_capacity(player, property_id, market);
    if debt_added < BID_INCREMENT || debt_added <= REFINANCE_FEE {
        return None;
    }
    let index = player
        .properties
        .iter()
        .position(|owned| owned.property.id == property_id)?;
    let cash_released = debt_added - REFINANCE_FEE;
    player.properties[index].debt += debt_added;
    player.debt += debt_added;
    player.cash += cash_released;
    Some(RefinanceResult {
        debt_added,
        fee: REFINANCE_FEE,
        cash_released,
    })
}

#[cfg(test)]
mod tests;
