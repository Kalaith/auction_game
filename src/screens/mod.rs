pub mod auction;
mod auction_lobby;
pub mod auction_widgets;
pub mod briefing;
pub mod dashboard;
pub mod esc_menu;
pub mod portfolio;
pub mod portfolio_widgets;
pub mod property_detail;
pub mod property_list;
pub mod sale_result;
pub mod title;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Screen {
    Title,
    Briefing,
    Dashboard,
    PropertyList,
    PropertyDetail(usize),
    Auction,
    Portfolio,
    SaleResult,
}
