use crate::model::{Auction, BidderActor};
use crate::sim::auction_sim::{push_log, AUCTION_DURATION_SECONDS};

pub(super) fn should_place_vendor_bid(auction: &Auction) -> bool {
    !auction.vendor_bid_used
        && auction.current_bid < auction.reserve_price
        && auction.current_bid < auction.property.guide_price
        && auction.seconds_remaining <= AUCTION_DURATION_SECONDS - 10.0
        && auction.seconds_remaining > 12.0
}

pub(super) fn place_vendor_bid(auction: &mut Auction) {
    let highest_legal_bid = auction.reserve_price - auction.bid_increment;
    let vendor_bid = auction.next_bid().min(highest_legal_bid);
    auction.vendor_bid_used = true;
    if vendor_bid <= auction.current_bid {
        return;
    }
    auction.current_bid = vendor_bid;
    auction.last_room_read = None;
    auction.last_bidder = Some(BidderActor::Vendor);
    push_log(
        auction,
        format!(
            "Auctioneer declares a vendor bid at {}.",
            crate::ui::format_money(vendor_bid)
        ),
    );
}

pub(super) fn announce_on_market(auction: &mut Auction) {
    if auction.on_market_announced || auction.current_bid < auction.reserve_price {
        return;
    }
    auction.on_market_announced = true;
    push_log(
        auction,
        "We are on the market. The highest bidder now buys the home.".to_string(),
    );
}
