pub mod auction;
pub mod auction_widgets;
pub mod dashboard;
pub mod portfolio;
pub mod portfolio_widgets;
pub mod property_detail;
pub mod property_list;
pub mod sale_result;

#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    Dashboard,
    PropertyList,
    PropertyDetail(usize),
    Auction,
    Portfolio,
    SaleResult,
}
