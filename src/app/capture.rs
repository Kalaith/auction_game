use super::App;
use crate::model::{OwnedProperty, Property, ResearchLevel, WalkawayStyle};
use crate::screens::Screen;
use crate::sim::rental::weekly_rent_for;
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
            "auction" => {
                self.start_new_game();
                if let Some(property) = self.available_properties.first().cloned() {
                    self.start_auction(property.id);
                }
            }
            "portfolio" => {
                self.start_new_game();
                self.seed_portfolio_capture();
                self.screen = Screen::Portfolio;
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
            let price = property.reserve_price;
            let property_deposit = deposit(price);
            let property_debt = price - property_deposit;
            let mut owned = OwnedProperty::new(
                property.clone(),
                price,
                purchase_fees(price),
                property_deposit,
                property_debt,
                price,
                ResearchLevel::BuildingInspection,
                WalkawayStyle::Balanced,
            );
            owned.is_leased = true;
            owned.weekly_rent = weekly_rent_for(&property, self.market());
            self.player.debt += property_debt;
            self.player.properties.push(owned);
        }
    }
}
