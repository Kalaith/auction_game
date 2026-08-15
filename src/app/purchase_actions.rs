use super::{App, PurchaseDebrief};
use crate::model::{Auction, AuctionStatus, OwnedProperty};
use crate::screens::Screen;
use crate::sim::finance::{finance_snapshot, rental_underwrite};
use crate::sim::valuation::{
    cash_needed_to_settle, deposit, market_adjusted_value, purchase_fees, sale_fees,
};
use crate::ui::format_money;

impl App {
    pub(crate) fn settle_purchase(&mut self) {
        let Some(auction) = self.current_auction.clone() else {
            return;
        };
        if auction.status != Some(AuctionStatus::SoldToPlayer) {
            return;
        }

        let purchase_price = auction.current_bid;
        let fees = purchase_fees(purchase_price);
        let deposit_paid = deposit(purchase_price);
        let cash_needed = cash_needed_to_settle(purchase_price);
        let finance = finance_snapshot(&self.player, self.market(), purchase_price);
        if self.player.cash < cash_needed || !finance.can_buy {
            self.status = format!(
                "Finance failed: {} cash after settle, {} bank headroom.",
                format_money(finance.cash_after_settle),
                format_money(finance.headroom_after)
            );
            return;
        }

        let debrief = self.purchase_debrief_for_auction(&auction);
        self.player.cash -= cash_needed;
        let property_debt = purchase_price - deposit_paid;
        self.player.debt += property_debt;
        let owned = OwnedProperty::new(
            auction.property.clone(),
            purchase_price,
            fees,
            deposit_paid,
            property_debt,
            auction.player_walkaway_price,
            auction.player_research_level,
            auction.walkaway_style,
        );
        self.purchase_debrief = Some(debrief);
        self.player.properties.push(owned);
        self.player.career.homes_bought += 1;
        if auction.sold_post_auction {
            self.player.career.post_auction_buys += 1;
        }
        self.portfolio_index = self.player.properties.len() - 1;
        self.available_properties
            .retain(|property| property.id != auction.property.id);
        self.current_auction = None;
        self.screen = Screen::Portfolio;
        self.status = "Property settled. Check the margin before buying upgrades.".to_string();
        self.refresh_campaign_outcome();
    }

    pub(crate) fn purchase_debrief_for_auction(&self, auction: &Auction) -> PurchaseDebrief {
        let estimated_resale = market_adjusted_value(&auction.property, self.market());
        let fees = purchase_fees(auction.current_bid) + sale_fees(estimated_resale);
        let cash_to_settle = cash_needed_to_settle(auction.current_bid);
        let cash_after_settle = self.player.cash - cash_to_settle;
        let walkaway_delta = auction.current_bid - auction.player_walkaway_price;
        let renovation_allowance = match auction.property.condition {
            crate::model::Condition::Rough => 42_000,
            crate::model::Condition::Tired => 18_000,
            crate::model::Condition::Solid => 9_000,
            crate::model::Condition::Premium => 4_000,
        };
        let renovation_allowance = match auction.property.deal_archetype {
            crate::model::DealArchetype::RiskyFixer => renovation_allowance + 18_000,
            crate::model::DealArchetype::RenovatorBait => renovation_allowance + 12_000,
            crate::model::DealArchetype::PrettyTrap => renovation_allowance + 8_000,
            crate::model::DealArchetype::LandValuePlay => renovation_allowance - 6_000,
            _ => renovation_allowance,
        }
        .max(0);
        let projected_profit = estimated_resale - auction.current_bid - fees - renovation_allowance;
        let contract_deposit = deposit(auction.current_bid);
        let loan_amount = auction.current_bid - contract_deposit;
        let rental = rental_underwrite(&auction.property, self.market(), auction.current_bid);
        let lesson = if walkaway_delta > 0 {
            "You won, but above your own walk-away line. That does not make the deal wrong, but it means the resale has less room to disappoint."
                .to_string()
        } else if cash_after_settle < 18_000 {
            "The price fits, but it leaves very little working cash. A cheap win can still become a cashflow squeeze."
                .to_string()
        } else if projected_profit < 0 {
            "The hammer price looks survivable until fees and repair allowance show up. Winning is not the same as buying well."
                .to_string()
        } else if projected_profit > 35_000 {
            "The bid leaves a real buffer after costs. This is the kind of margin that lets you choose renovations instead of needing them."
                .to_string()
        } else {
            "The bid leaves a workable margin before renovations. The next decision is whether upgrades improve that margin or eat it."
                .to_string()
        };

        PurchaseDebrief {
            address: auction.property.address.clone(),
            purchase_price: auction.current_bid,
            estimated_resale,
            fees,
            cash_to_settle,
            cash_after_settle,
            renovation_allowance,
            walkaway_delta,
            projected_profit,
            contract_deposit,
            loan_amount,
            weekly_rent: rental.gross_rent,
            weekly_rental_cashflow: rental.net_cashflow,
            lesson,
        }
    }
}
