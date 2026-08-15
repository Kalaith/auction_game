use crate::app::{App, PurchaseDebrief};
use crate::model::{
    Auction, CampaignStatus, ContractorTier, Player, PropertyId, ResearchReport, WalkawayStyle,
};
use crate::screens::Screen;
use crate::sim::sale_sim::{MarketingPlan, SaleResult};
use macroquad::prelude::set_fullscreen;
use macroquad_toolkit::persistence::{load_from_slot, save_to_slot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const GAME_NAME: &str = "auction_house_tycoon";
const QUICK_SLOT: &str = "quicksave";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SaveGameState {
    player: Player,
    week: u32,
    market_index: usize,
    screen: Screen,
    current_auction: Option<Auction>,
    purchase_debrief: Option<PurchaseDebrief>,
    sale_result: Option<SaleResult>,
    research_reports: HashMap<PropertyId, ResearchReport>,
    selected_contractor: ContractorTier,
    #[serde(default)]
    selected_marketing_plan: MarketingPlan,
    campaign_status: CampaignStatus,
    listing_filter: usize,
    #[serde(default)]
    portfolio_index: usize,
    walkaway_price: i64,
    walkaway_style: WalkawayStyle,
    status: String,
    fullscreen_enabled: bool,
}

impl SaveGameState {
    fn from_app(app: &App) -> Self {
        Self {
            player: app.player.clone(),
            week: app.week,
            market_index: app.market_index,
            screen: app.screen.clone(),
            current_auction: app.current_auction.clone(),
            purchase_debrief: app.purchase_debrief.clone(),
            sale_result: app.sale_result.clone(),
            research_reports: app.research_reports.clone(),
            selected_contractor: app.selected_contractor,
            selected_marketing_plan: app.selected_marketing_plan,
            campaign_status: app.campaign_status,
            listing_filter: app.listing_filter,
            portfolio_index: app.portfolio_index,
            walkaway_price: app.walkaway_price,
            walkaway_style: app.walkaway_style,
            status: app.status.clone(),
            fullscreen_enabled: app.fullscreen_enabled,
        }
    }
}

impl App {
    pub(crate) fn save_game(&mut self) {
        let save = SaveGameState::from_app(self);
        self.status = match save_to_slot(GAME_NAME, QUICK_SLOT, &save) {
            Ok(()) => "Game saved.".to_string(),
            Err(error) => format!("Save failed: {error}"),
        };
    }

    pub(crate) fn load_game(&mut self) {
        match self.apply_saved_game() {
            Ok(()) => {
                self.esc_menu_open = false;
                self.esc_settings_open = false;
                self.status = "Game loaded.".to_string();
            }
            Err(error) => {
                self.status = format!("Load failed: {error}");
            }
        }
    }

    pub(crate) fn load_game_from_title(&mut self) {
        let _ = self.apply_saved_game();
    }

    fn apply_saved_game(&mut self) -> Result<(), String> {
        let save: SaveGameState = load_from_slot(GAME_NAME, QUICK_SLOT)?;
        self.player = save.player;
        self.week = save.week.max(1);
        self.market_index = save.market_index.min(self.data.market_events.len() - 1);
        self.screen = save.screen;
        self.current_auction = save.current_auction;
        self.purchase_debrief = save.purchase_debrief;
        self.sale_result = save.sale_result;
        self.research_reports = save.research_reports;
        self.selected_contractor = save.selected_contractor;
        self.selected_marketing_plan = save.selected_marketing_plan;
        self.campaign_status = save.campaign_status;
        self.listing_filter = save.listing_filter;
        self.portfolio_index = save
            .portfolio_index
            .min(self.player.properties.len().saturating_sub(1));
        self.walkaway_price = save.walkaway_price;
        self.walkaway_style = save.walkaway_style;
        self.status = save.status;
        self.fullscreen_enabled = save.fullscreen_enabled;
        set_fullscreen(self.fullscreen_enabled);
        self.title_settings_open = false;
        self.esc_menu_open = false;
        self.esc_settings_open = false;
        self.refresh_available_properties();

        if self.screen == Screen::Title {
            self.screen = Screen::Dashboard;
        }
        if self.screen == Screen::Auction && self.current_auction.is_none() {
            self.screen = Screen::PropertyList;
        }
        Ok(())
    }
}
