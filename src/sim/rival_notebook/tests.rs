use super::*;
use crate::data::GameData;
use crate::model::{AuctionStatus, ResearchLevel, WalkawayStyle};
use crate::sim::auction_sim::create_auction;

#[test]
fn completed_rooms_accumulate_recurring_rival_history() {
    let data = GameData::load();
    let property = crate::model::Property::from_template(&data.properties[0]);
    let mut auction = create_auction(
        &property,
        &data.market_events[0],
        &data.bidder_profiles,
        property.reserve_price,
        ResearchLevel::StreetScan,
        WalkawayStyle::Balanced,
    );
    let winner = auction.bidders[0].name.clone();
    auction.current_bid = property.reserve_price + 20_000;
    auction.bidders[0].stretch_bid_used = true;
    auction.status = Some(AuctionStatus::SoldToNpc(winner.clone()));
    let mut notebook = Vec::new();

    record_completed_room(&mut notebook, &auction);
    record_completed_room(&mut notebook, &auction);

    let record = notebook
        .iter()
        .find(|record| record.name == winner)
        .expect("winner should enter the notebook");
    assert_eq!(record.auctions_met, 2);
    assert_eq!(record.auctions_won, 2);
    assert_eq!(record.stretches_seen, 2);
    assert_eq!(record.highest_room_price, auction.current_bid);
}

#[test]
fn an_unfinished_lobby_does_not_create_rival_knowledge() {
    let data = GameData::load();
    let property = crate::model::Property::from_template(&data.properties[0]);
    let auction = create_auction(
        &property,
        &data.market_events[0],
        &data.bidder_profiles,
        property.reserve_price,
        ResearchLevel::StreetScan,
        WalkawayStyle::Balanced,
    );
    let mut notebook = Vec::new();

    record_completed_room(&mut notebook, &auction);

    assert!(notebook.is_empty());
}
