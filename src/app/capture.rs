use super::App;
use crate::model::{
    AuctionStatus, AuctionTemperature, BidderActor, BidderMood, CampaignStatus, OwnedProperty,
    Property, ResearchLevel, RivalRecord, WalkawayStyle,
};
use crate::screens::Screen;
use crate::sim::auction_sim::{begin_auction_calls, hold_player_position, place_player_bid};
use crate::sim::maintenance::{next_maintenance_week, trigger_due_maintenance};
use crate::sim::rental::weekly_rent_for_owned;
use crate::sim::sale_sim::{simulate_sale, MarketingPlan, ReserveChoice};
use crate::sim::valuation::{deposit, purchase_fees};

impl App {
    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "briefing" => self.start_new_game(),
            "dashboard" => {
                self.start_new_game();
                self.screen = Screen::Dashboard;
                self.status =
                    "Choose a listing worth one of this week's registrations.".to_string();
            }
            "listings" => {
                self.start_new_game();
                self.screen = Screen::PropertyList;
            }
            "detail" => {
                self.start_new_game();
                self.open_property_detail(0);
            }
            "detail_full" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.open_property_detail(0);
                    self.buy_research(property.id, ResearchLevel::FullDiligence);
                }
            }
            "detail_compact" => self.seed_property_detail_capture(6),
            "detail_premium" => self.seed_property_detail_capture(4),
            "detail_large_block" => self.seed_property_detail_capture(11),
            "auction" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.start_auction(property.id);
                    if let Some(auction) = self.current_auction.as_mut() {
                        begin_auction_calls(auction);
                    }
                    self.status =
                        "Bidding is live. Read the room and protect the walk-away.".to_string();
                }
            }
            "auction_lobby" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.start_auction(property.id);
                }
            }
            "auction_on_market" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.start_auction(property.id);
                    if let Some(auction) = self.current_auction.as_mut() {
                        begin_auction_calls(auction);
                        auction.current_bid = auction.reserve_price - auction.bid_increment;
                        place_player_bid(auction);
                    }
                }
            }
            "auction_final_call" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.start_auction(property.id);
                    if let Some(auction) = self.current_auction.as_mut() {
                        begin_auction_calls(auction);
                        auction.current_bid = auction.reserve_price - auction.bid_increment;
                        place_player_bid(auction);
                        auction.seconds_remaining = 6.0;
                        auction.temperature = AuctionTemperature::FinalCall;
                    }
                    self.status = "Your paddle leads at the final call. Tap a visible action now."
                        .to_string();
                }
            }
            "auction_won" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.start_auction(property.id);
                    if let Some(auction) = self.current_auction.as_mut() {
                        begin_auction_calls(auction);
                        auction.current_bid = auction.reserve_price;
                        auction.last_bidder = Some(BidderActor::Player);
                        auction.on_market_announced = true;
                        auction.status = Some(AuctionStatus::SoldToPlayer);
                    }
                    self.status =
                        "Hammer down. Review the deposit, loan, rent, and cashflow before settling."
                            .to_string();
                }
            }
            "auction_read" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.start_auction(property.id);
                    if let Some(auction) = self.current_auction.as_ref() {
                        self.player.rival_notebook = auction
                            .bidders
                            .iter()
                            .map(|bidder| RivalRecord {
                                name: bidder.name.clone(),
                                bidder_type: bidder.bidder_type,
                                auctions_met: 2,
                                auctions_won: u32::from(
                                    bidder.bidder_type == crate::model::BidderType::Investor,
                                ),
                                highest_room_price: 690_000,
                                stretches_seen: u32::from(
                                    bidder.bidder_type == crate::model::BidderType::FirstHomeBuyer,
                                ),
                            })
                            .collect();
                    }
                    if let Some(auction) = self.current_auction.as_mut() {
                        begin_auction_calls(auction);
                        hold_player_position(auction);
                    }
                }
            }
            "auction_stretch" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.start_auction(property.id);
                    if let Some(auction) = self.current_auction.as_mut() {
                        begin_auction_calls(auction);
                        auction.temperature = AuctionTemperature::FomoSpiral;
                        auction.seconds_remaining = 14.0;
                        auction.call_timer = 99.0;
                        for bidder in &mut auction.bidders {
                            bidder.reaction_timer = 99.0;
                        }
                        if let Some(index) = auction.bidders.iter().position(|bidder| {
                            bidder.bidder_type == crate::model::BidderType::Renovator
                        }) {
                            let bidder = &mut auction.bidders[index];
                            bidder.stretch_bid_used = true;
                            bidder.mood = BidderMood::Stretching;
                            bidder.heat = 94;
                            bidder.tell =
                                "Upside fantasy is outrunning the repair budget.".to_string();
                            auction.current_bid = bidder.max_price + auction.bid_increment;
                            auction.last_bidder = Some(BidderActor::Player);
                            auction.last_room_read = Some(format!(
                                "{} has crossed the repair budget; another bid may be emotional.",
                                bidder.name
                            ));
                        }
                    }
                    self.status = "A rival is stretching. Read the cause before chasing the room."
                        .to_string();
                }
            }
            "passed_in" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.start_auction(property.id);
                    if let Some(auction) = self.current_auction.as_mut() {
                        begin_auction_calls(auction);
                        auction.current_bid = auction.reserve_price - auction.bid_increment * 3;
                        auction.status = Some(AuctionStatus::PassedIn);
                        auction.player_research_level = ResearchLevel::FullDiligence;
                    }
                }
            }
            "portfolio" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                self.screen = Screen::Portfolio;
                self.status =
                    "Compare the selected home's cashflow, debt, condition, and next move."
                        .to_string();
            }
            "dashboard_weekly" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                self.screen = Screen::Dashboard;
                self.advance_week();
                self.screen = Screen::Dashboard;
                self.status = "Week closed. Compare rent against every portfolio cost.".to_string();
            }
            "portfolio_maintenance" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                if let Some(owned) = self.player.properties.first_mut() {
                    owned.weeks_held = next_maintenance_week(owned);
                }
                trigger_due_maintenance(&mut self.player);
                self.screen = Screen::Portfolio;
                self.status = "A maintenance check found an issue. Repair it to restore full rent."
                    .to_string();
            }
            "portfolio_refinance" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                if let Some(owned) = self.player.properties.first_mut() {
                    owned.weeks_held = 4;
                    owned.debt -= 70_000;
                    self.player.debt -= 70_000;
                }
                self.screen = Screen::Portfolio;
                self.status =
                    "Seasoned equity is available. Releasing it raises debt and funds the next deposit."
                        .to_string();
            }
            "portfolio_refinanced" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                if let Some(owned) = self.player.properties.first_mut() {
                    owned.weeks_held = 4;
                    owned.debt -= 70_000;
                    self.player.debt -= 70_000;
                }
                if let Some(property_id) = self
                    .player
                    .properties
                    .first()
                    .map(|owned| owned.property.id)
                {
                    self.refinance_owned_property(property_id);
                }
                self.screen = Screen::Portfolio;
            }
            "portfolio_paydown" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                if let Some(property_id) = self
                    .player
                    .properties
                    .first()
                    .map(|owned| owned.property.id)
                {
                    self.pay_down_property_debt(property_id);
                }
                self.screen = Screen::Portfolio;
            }
            "portfolio_leasing" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                if let Some(owned) = self.player.properties.first_mut() {
                    owned.is_leased = false;
                    owned.leasing_weeks_remaining = 1;
                }
                self.screen = Screen::Portfolio;
                self.status = "The letting fee is paid. Tap HOLD to close the vacant leasing week."
                    .to_string();
            }
            "portfolio_review" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                if let Some(owned) = self.player.properties.first_mut() {
                    owned.weeks_held = 9;
                    owned.next_rent_review_week = 9;
                }
                self.screen = Screen::Portfolio;
                self.status = "Rent review due. Renew safely or test the market and risk vacancy."
                    .to_string();
            }
            "sale_result" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                if let Some(owned) = self.player.properties.first().cloned() {
                    self.sale_result = Some(simulate_sale(
                        &owned,
                        self.market(),
                        ReserveChoice::Conservative,
                        MarketingPlan::Standard,
                    ));
                    self.screen = Screen::SaleResult;
                    self.status = "Sale settled. Read how much capital returned to the portfolio."
                        .to_string();
                }
            }
            "conclusion" => {
                self.start_new_game();
                let properties: Vec<Property> = self
                    .data
                    .properties
                    .iter()
                    .filter(|template| [2, 6, 8].contains(&template.id))
                    .map(Property::from_template)
                    .collect();
                for property in properties {
                    self.seed_owned_property(property);
                }
                self.player.cash = 300_000;
                self.player.career.auctions_attended = 9;
                self.player.career.homes_bought = 3;
                self.player.career.disciplined_walkaways = 4;
                self.player.career.post_auction_buys = 1;
                self.player.career.homes_sold = 1;
                self.player.career.realized_profit = 38_000;
                self.player.career.unused_registrations = 7;
                self.auction_registrations = 0;
                self.campaign_status = CampaignStatus::Won;
                self.screen = Screen::Dashboard;
                self.status =
                    "Campaign complete. Review the portfolio or start a new season.".to_string();
            }
            "conclusion_failed" => {
                self.start_new_game();
                let properties: Vec<Property> = self
                    .data
                    .properties
                    .iter()
                    .filter(|template| [6, 8].contains(&template.id))
                    .map(Property::from_template)
                    .collect();
                for property in properties {
                    self.seed_owned_property(property);
                }
                self.player.cash = 28_000;
                self.player.career.auctions_attended = 11;
                self.player.career.homes_bought = 2;
                self.player.career.disciplined_walkaways = 5;
                self.player.career.unused_registrations = 12;
                self.auction_registrations = 0;
                self.week = 25;
                self.campaign_status = CampaignStatus::Failed;
                self.screen = Screen::Dashboard;
                self.status =
                    "Season closed. Read the binding constraint before starting again.".to_string();
            }
            "title" => {}
            _ => {
                self.start_new_game();
                self.screen = Screen::Dashboard;
            }
        }
    }

    fn seed_portfolio_capture(&mut self) {
        let samples: Vec<Property> = self
            .data
            .properties
            .iter()
            .map(Property::from_template)
            .filter(|property| property.hidden_defect_risk < 0.18)
            .take(3)
            .collect();
        for property in samples {
            self.seed_owned_property(property);
        }
    }

    fn seed_property_detail_capture(&mut self, property_id: usize) {
        self.start_new_game();
        if let Some(template) = self
            .data
            .properties
            .iter()
            .find(|template| template.id == property_id)
        {
            self.available_properties = vec![Property::from_template(template)];
            self.open_property_detail(0);
            self.status =
                "Compare the guide, researched range, rent, margin, and cash before registering."
                    .to_string();
        }
    }

    fn seed_owned_property(&mut self, property: Property) {
        let price = property.reserve_price;
        let property_deposit = deposit(price);
        let property_debt = price - property_deposit;
        let mut owned = OwnedProperty::new(
            property,
            price,
            purchase_fees(price),
            property_deposit,
            property_debt,
            price,
            ResearchLevel::BuildingInspection,
            WalkawayStyle::Balanced,
        );
        owned.is_leased = true;
        owned.weekly_rent = weekly_rent_for_owned(&owned, self.market());
        self.player.debt += property_debt;
        self.player.properties.push(owned);
    }
}
