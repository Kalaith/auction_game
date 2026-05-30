use crate::data::GameData;
use crate::model::{
    AuctionStatus, CampaignStatus, ContractorTier, OwnedProperty, Player, Property, PropertyId,
    ResearchLevel, ResearchReport, WalkawayStyle,
};
use crate::screens::Screen;
use crate::sim::auction_sim::{create_auction, update_auction};
use crate::sim::campaign::{
    apply_weekly_pressure, campaign_status, next_unlock_note, suburb_is_unlocked,
};
use crate::sim::finance::finance_snapshot;
use crate::sim::renovation::{
    progress_player_renovations, quote_renovation, start_upgrade_project,
};
use crate::sim::research::{recommended_walkaway, research_cost};
use crate::sim::sale_sim::{simulate_sale, MarketingPlan, ReserveChoice, SaleResult};
use crate::sim::valuation::{
    cash_needed_to_settle, deposit, market_adjusted_value, net_worth, purchase_fees, sale_fees,
};
use crate::ui::*;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct PurchaseDebrief {
    pub(crate) address: String,
    pub(crate) purchase_price: i64,
    pub(crate) estimated_resale: i64,
    pub(crate) fees: i64,
    pub(crate) cash_to_settle: i64,
    pub(crate) cash_after_settle: i64,
    pub(crate) renovation_allowance: i64,
    pub(crate) walkaway_delta: i64,
    pub(crate) projected_profit: i64,
    pub(crate) lesson: String,
}

pub struct App {
    pub(crate) title_background: Texture2D,
    pub(crate) title_settings_open: bool,
    pub(crate) esc_menu_open: bool,
    pub(crate) esc_settings_open: bool,
    pub(crate) fullscreen_enabled: bool,
    pub(crate) data: GameData,
    pub(crate) player: Player,
    pub(crate) available_properties: Vec<Property>,
    pub(crate) week: u32,
    pub(crate) market_index: usize,
    pub(crate) screen: Screen,
    pub(crate) current_auction: Option<crate::model::Auction>,
    pub(crate) purchase_debrief: Option<PurchaseDebrief>,
    pub(crate) sale_result: Option<SaleResult>,
    pub(crate) research_reports: HashMap<PropertyId, ResearchReport>,
    pub(crate) selected_contractor: ContractorTier,
    pub(crate) selected_marketing_plan: MarketingPlan,
    pub(crate) campaign_status: CampaignStatus,
    pub(crate) listing_filter: usize,
    pub(crate) walkaway_price: i64,
    pub(crate) walkaway_style: WalkawayStyle,
    pub(crate) status: String,
}

impl App {
    pub fn new(title_background: Texture2D) -> Self {
        let data = GameData::load();
        let mut app = Self {
            title_background,
            title_settings_open: false,
            esc_menu_open: false,
            esc_settings_open: false,
            fullscreen_enabled: false,
            data,
            player: Player::new(),
            available_properties: Vec::new(),
            week: 1,
            market_index: 0,
            screen: Screen::Title,
            current_auction: None,
            purchase_debrief: None,
            sale_result: None,
            research_reports: HashMap::new(),
            selected_contractor: ContractorTier::Reliable,
            selected_marketing_plan: MarketingPlan::Standard,
            campaign_status: CampaignStatus::Active,
            listing_filter: 0,
            walkaway_price: 600_000,
            walkaway_style: WalkawayStyle::Balanced,
            status: "Read the market, pick a property, and keep your margin alive.".to_string(),
        };
        app.refresh_available_properties();
        app
    }

    pub fn update(&mut self, dt: f32) {
        if self.screen != Screen::Title && is_key_pressed(KeyCode::Escape) {
            if self.esc_settings_open {
                self.esc_settings_open = false;
            } else {
                self.esc_menu_open = !self.esc_menu_open;
            }
        }

        if !self.esc_menu_open && self.screen == Screen::Auction {
            if let Some(auction) = self.current_auction.as_mut() {
                update_auction(auction, dt);
            }
        }
    }

    pub fn draw(&mut self) {
        clear_background(BACKGROUND);
        begin_ui_frame();

        if self.screen == Screen::Title {
            self.draw_title_screen();
            return;
        }

        set_ui_input_enabled(!self.esc_menu_open);
        self.draw_header();

        match self.screen.clone() {
            Screen::Title => self.draw_title_screen(),
            Screen::Dashboard => self.draw_dashboard(),
            Screen::PropertyList => self.draw_property_list(),
            Screen::PropertyDetail(index) => self.draw_property_detail(index),
            Screen::Auction => self.draw_auction(),
            Screen::Portfolio => self.draw_portfolio(),
            Screen::SaleResult => self.draw_sale_result(),
        }

        self.draw_status_bar();
        set_ui_input_enabled(true);

        if self.esc_menu_open {
            self.draw_esc_menu();
        }
    }

    pub(crate) fn market(&self) -> &crate::model::MarketEvent {
        &self.data.market_events[self.market_index]
    }

    pub(crate) fn research_level(&self, property_id: PropertyId) -> ResearchLevel {
        self.research_reports
            .get(&property_id)
            .map(|report| report.level)
            .unwrap_or(ResearchLevel::StreetScan)
    }

    fn draw_header(&mut self) {
        draw_rectangle(0.0, 0.0, ui_width(), 68.0, PANEL_DARK);
        if button(
            Rect::new(16.0, 16.0, 36.0, 36.0),
            "\u{2699}",
            true,
            ButtonTone::Ghost,
        ) {
            self.esc_menu_open = !self.esc_menu_open;
            self.esc_settings_open = false;
        }
        label("Auction House Tycoon", 68.0, 42.0, 30, TEXT_BRIGHT);
        label(&format!("Week {}", self.week), 430.0, 41.0, 19, TEXT_DIM);

        let mut x = ui_width() - 422.0;
        let nav_enabled = self
            .current_auction
            .as_ref()
            .map(|auction| !auction.is_running())
            .unwrap_or(true);

        if button(
            Rect::new(x, 18.0, 130.0, 34.0),
            "Dashboard",
            nav_enabled,
            ButtonTone::Ghost,
        ) {
            self.screen = Screen::Dashboard;
        }
        x += 142.0;
        if button(
            Rect::new(x, 18.0, 120.0, 34.0),
            "Listings",
            nav_enabled,
            ButtonTone::Ghost,
        ) {
            self.screen = Screen::PropertyList;
        }
        x += 132.0;
        if button(
            Rect::new(x, 18.0, 120.0, 34.0),
            "Portfolio",
            nav_enabled,
            ButtonTone::Ghost,
        ) {
            self.screen = Screen::Portfolio;
        }
    }

    fn draw_status_bar(&self) {
        let rect = Rect::new(0.0, ui_height() - 40.0, ui_width(), 40.0);
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_DARK);
        label(&self.status, 28.0, ui_height() - 15.0, 17, TEXT_DIM);
    }

    pub(crate) fn open_property_detail(&mut self, index: usize) {
        if let Some(property) = self.available_properties.get(index) {
            self.walkaway_price = recommended_walkaway(
                property,
                self.market(),
                self.research_level(property.id),
                self.walkaway_style,
                self.player.reputation,
            );
            self.screen = Screen::PropertyDetail(index);
        }
    }

    pub(crate) fn start_new_game(&mut self) {
        self.player = Player::new();
        self.available_properties.clear();
        self.week = 1;
        self.market_index = 0;
        self.screen = Screen::Dashboard;
        self.current_auction = None;
        self.purchase_debrief = None;
        self.sale_result = None;
        self.research_reports.clear();
        self.selected_contractor = ContractorTier::Reliable;
        self.selected_marketing_plan = MarketingPlan::Standard;
        self.campaign_status = CampaignStatus::Active;
        self.listing_filter = 0;
        self.walkaway_price = 600_000;
        self.walkaway_style = WalkawayStyle::Balanced;
        self.status = "Read the market, pick a property, and keep your margin alive.".to_string();
        self.title_settings_open = false;
        self.esc_menu_open = false;
        self.esc_settings_open = false;
        self.refresh_available_properties();
    }

    pub(crate) fn return_to_title(&mut self) {
        self.screen = Screen::Title;
        self.title_settings_open = false;
        self.esc_menu_open = false;
        self.esc_settings_open = false;
    }

    pub(crate) fn toggle_fullscreen(&mut self) {
        self.fullscreen_enabled = !self.fullscreen_enabled;
        set_fullscreen(self.fullscreen_enabled);
    }

    pub(crate) fn buy_research(&mut self, property_id: PropertyId, level: ResearchLevel) {
        let current = self.research_level(property_id);
        if level <= current {
            return;
        }

        let cost = research_cost(level, self.player.reputation);
        if self.player.cash < cost {
            self.status = format!("Need {} for {}.", format_money(cost), level.label());
            return;
        }

        let Some(property) = self
            .available_properties
            .iter()
            .find(|property| property.id == property_id)
            .cloned()
        else {
            return;
        };

        self.player.cash -= cost;
        self.research_reports
            .insert(property_id, ResearchReport { level });
        self.walkaway_price = recommended_walkaway(
            &property,
            self.market(),
            level,
            self.walkaway_style,
            self.player.reputation,
        );
        self.status = format!(
            "{} complete for {}. Walk-away updated to {}.",
            level.label(),
            property.address,
            format_money(self.walkaway_price)
        );
    }

    pub(crate) fn start_auction(&mut self, property_id: PropertyId) {
        let Some(property) = self
            .available_properties
            .iter()
            .find(|property| property.id == property_id)
            .cloned()
        else {
            return;
        };
        self.current_auction = Some(create_auction(
            &property,
            self.market(),
            &self.data.bidder_profiles,
            self.walkaway_price,
            self.research_level(property.id),
            self.walkaway_style,
        ));
        self.purchase_debrief = None;
        self.screen = Screen::Auction;
        self.status =
            "Auction started. Bid deliberately; losing can be a good decision.".to_string();
    }

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
        self.available_properties
            .retain(|property| property.id != auction.property.id);
        self.current_auction = None;
        self.screen = Screen::Portfolio;
        self.status = "Property settled. Check the margin before buying upgrades.".to_string();
    }

    pub(crate) fn buy_upgrade(&mut self, property_id: PropertyId, upgrade_id: &str) {
        let Some(index) = self
            .player
            .properties
            .iter()
            .position(|owned| owned.property.id == property_id)
        else {
            return;
        };
        let Some(upgrade) = self
            .data
            .upgrades
            .iter()
            .find(|upgrade| upgrade.id == upgrade_id)
        else {
            return;
        };
        if self.player.properties[index].has_upgrade(upgrade_id)
            || self.player.properties[index].has_active_upgrade(upgrade_id)
        {
            return;
        }
        if self.player.properties[index].active_renovation.is_some() {
            self.status = "Finish the current renovation before starting another.".to_string();
            return;
        }

        let quote = quote_renovation(
            &self.player.properties[index],
            upgrade,
            self.selected_contractor,
            self.market(),
            self.player.reputation,
        );
        if self.player.cash < quote.total_cost {
            self.status = "Not enough cash to start that renovation.".to_string();
            return;
        }

        self.player.cash -= quote.total_cost;
        let project = start_upgrade_project(&quote, self.week, self.player.reputation);
        self.player.properties[index].active_renovation = Some(project.clone());
        self.status = format!(
            "{} started: {} week job, {} paid. Advance weeks to finish.",
            upgrade.name,
            project.weeks_total,
            format_money(quote.total_cost)
        );
    }

    pub(crate) fn sell_property(&mut self, property_id: PropertyId, choice: ReserveChoice) {
        let Some(index) = self
            .player
            .properties
            .iter()
            .position(|owned| owned.property.id == property_id)
        else {
            return;
        };
        if self.player.properties[index].active_renovation.is_some() {
            self.status = "Finish the active renovation before selling.".to_string();
            return;
        }
        let owned = self.player.properties[index].clone();
        let marketing_plan = self.selected_marketing_plan;
        let marketing_cost = marketing_plan.cost();
        if self.player.cash < marketing_cost {
            self.status = format!(
                "Need {} for the {} campaign.",
                format_money(marketing_cost),
                marketing_plan.label()
            );
            return;
        }

        self.player.cash -= marketing_cost;
        let result = simulate_sale(&owned, self.market(), choice, marketing_plan);

        if let Some(sale_price) = result.sale_price {
            self.player.cash += sale_price - result.selling_fees - owned.debt;
            self.player.debt -= owned.debt;
            self.player.reputation += result.reputation_delta;
            self.player.properties.remove(index);
            self.status = format!(
                "{} sale complete with {} {}. {}",
                choice.label(),
                if result.profit >= 0 { "profit" } else { "loss" },
                format_money(result.profit.abs()),
                result.reputation_reason
            );
        } else {
            let holding = self.player.properties[index].property.holding_cost_per_week;
            self.player.cash -= holding;
            self.player.properties[index].weeks_held += 1;
            self.status = "Auction passed in. One more week of holding costs was paid.".to_string();
        }

        self.sale_result = Some(result);
        let current_net_worth = net_worth(&self.player, self.market());
        self.campaign_status = campaign_status(self.week, current_net_worth);
        if self.campaign_status == CampaignStatus::Won {
            self.status = format!(
                "Campaign won with {} net worth. Review the sale result.",
                format_money(current_net_worth)
            );
        }
        self.screen = Screen::SaleResult;
    }

    pub(crate) fn purchase_debrief_for_auction(
        &self,
        auction: &crate::model::Auction,
    ) -> PurchaseDebrief {
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
            lesson,
        }
    }

    pub(crate) fn advance_week(&mut self) {
        if self.campaign_status.is_finished() {
            self.status = format!("{}.", self.campaign_status.label());
            return;
        }

        let market = self.market().clone();
        let pressure = apply_weekly_pressure(&mut self.player, &market);
        let completed_jobs = progress_player_renovations(&mut self.player);
        self.week += 1;
        self.market_index = ((self.week - 1) as usize) % self.data.market_events.len();
        let current_net_worth = net_worth(&self.player, self.market());
        self.campaign_status = campaign_status(self.week, current_net_worth);
        self.refresh_available_properties();
        self.status = match self.campaign_status {
            CampaignStatus::Won => format!(
                "Campaign won in week {} with {} net worth.",
                self.week,
                format_money(current_net_worth)
            ),
            CampaignStatus::Failed => format!(
                "Campaign failed after week 52. Final net worth: {}.",
                format_money(current_net_worth)
            ),
            CampaignStatus::Active => {
                let pressure_note = if pressure.total > 0 {
                    format!(
                        "Paid {} carrying costs ({} holding, {} interest).",
                        format_money(pressure.total),
                        format_money(pressure.holding_cost),
                        format_money(pressure.debt_interest)
                    )
                } else {
                    "No carrying costs this week.".to_string()
                };
                let cashflow_note = if pressure.shortfall_added_to_debt > 0 {
                    format!(
                        " Cash shortfall added {} to debt.",
                        format_money(pressure.shortfall_added_to_debt)
                    )
                } else {
                    String::new()
                };
                let renovation_note = if completed_jobs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", completed_jobs.join(" "))
                };
                format!(
                    "Week {} market pulse. {}{}{} {}",
                    self.week,
                    pressure_note,
                    cashflow_note,
                    renovation_note,
                    next_unlock_note(self.week, current_net_worth, self.player.reputation)
                )
            }
        };
    }

    pub(crate) fn refresh_available_properties(&mut self) {
        let current_net_worth = net_worth(&self.player, self.market());
        let owned_ids: Vec<PropertyId> = self
            .player
            .properties
            .iter()
            .map(|owned| owned.property.id)
            .collect();
        self.available_properties = self
            .data
            .properties
            .iter()
            .map(Property::from_template)
            .filter(|property| !owned_ids.contains(&property.id))
            .filter(|property| {
                suburb_is_unlocked(
                    &property.suburb,
                    self.week,
                    current_net_worth,
                    self.player.reputation,
                )
            })
            .collect();
    }
}
