use super::App;
use crate::model::{CampaignStatus, OwnedProperty, Property, ResearchLevel, WalkawayStyle};
use crate::screens::Screen;
use crate::sim::auction_sim::{hold_player_position, place_player_bid};
use crate::sim::maintenance::{next_maintenance_week, trigger_due_maintenance};
use crate::sim::rental::weekly_rent_for_owned;
use crate::sim::valuation::{deposit, purchase_fees};

impl App {
    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "briefing" => self.start_new_game(),
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
            "auction" => {
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
                        auction.current_bid = auction.reserve_price - auction.bid_increment;
                        place_player_bid(auction);
                    }
                }
            }
            "auction_read" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.start_auction(property.id);
                    if let Some(auction) = self.current_auction.as_mut() {
                        hold_player_position(auction);
                    }
                }
            }
            "portfolio" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                self.screen = Screen::Portfolio;
            }
            "portfolio_maintenance" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                if let Some(owned) = self.player.properties.first_mut() {
                    owned.weeks_held = next_maintenance_week(owned);
                }
                trigger_due_maintenance(&mut self.player);
                self.screen = Screen::Portfolio;
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
                self.campaign_status = CampaignStatus::Won;
                self.screen = Screen::Dashboard;
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
            .available_properties
            .iter()
            .filter(|property| property.hidden_defect_risk < 0.18)
            .take(2)
            .cloned()
            .collect();
        for property in samples {
            self.seed_owned_property(property);
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
