use crate::model::{Auction, AuctionStatus, BidderActor};
use crate::sim::auction_sim::{push_log, AUCTION_DURATION_SECONDS};

pub const ON_MARKET_BID_INCREMENT: i64 = 5_000;

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
    auction.bid_increment = ON_MARKET_BID_INCREMENT;
    push_log(
        auction,
        "We are on the market. The highest bidder buys; the auctioneer now takes $5,000 rises."
            .to_string(),
    );
}

pub fn post_auction_offer(auction: &Auction) -> Option<i64> {
    if auction.status != Some(AuctionStatus::PassedIn) {
        return None;
    }
    Some(
        (auction.reserve_price - auction.bid_increment)
            .max(auction.current_bid)
            .min(auction.reserve_price),
    )
}

pub fn accept_post_auction_offer(auction: &mut Auction) -> bool {
    let Some(offer) = post_auction_offer(auction) else {
        return false;
    };
    auction.current_bid = offer;
    auction.last_bidder = Some(BidderActor::Player);
    auction.status = Some(AuctionStatus::SoldToPlayer);
    auction.sold_post_auction = true;
    push_log(
        auction,
        format!(
            "Vendor accepts your post-auction offer of {}.",
            crate::ui::format_money(offer)
        ),
    );
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostAuctionTestResult {
    Accepted(i64),
    Rejected(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VendorStance {
    Flexible,
    Negotiable,
    Firm,
}

impl VendorStance {
    pub fn label(self) -> &'static str {
        match self {
            VendorStance::Flexible => "Flexible seller · a lower test has support",
            VendorStance::Negotiable => "Negotiable seller · a modest test may work",
            VendorStance::Firm => "Firm seller · a low test risks rejection",
        }
    }
}

pub fn vendor_stance(auction: &Auction) -> VendorStance {
    if auction.property.hidden_defect_risk >= 0.55 || auction.property.buyer_demand <= 50 {
        VendorStance::Flexible
    } else if auction.property.buyer_demand <= 70 {
        VendorStance::Negotiable
    } else {
        VendorStance::Firm
    }
}

pub fn test_vendor_at_passed_in_price(auction: &mut Auction) -> Option<PostAuctionTestResult> {
    if auction.status != Some(AuctionStatus::PassedIn) || auction.post_auction_tested {
        return None;
    }
    auction.post_auction_tested = true;
    let offer = auction.current_bid;
    if offer >= vendor_acceptance_floor(auction) {
        auction.last_bidder = Some(BidderActor::Player);
        auction.status = Some(AuctionStatus::SoldToPlayer);
        auction.sold_post_auction = true;
        push_log(
            auction,
            format!(
                "Vendor accepts your passed-in offer of {}.",
                crate::ui::format_money(offer)
            ),
        );
        Some(PostAuctionTestResult::Accepted(offer))
    } else {
        let counter = post_auction_offer(auction).unwrap_or(auction.reserve_price);
        push_log(
            auction,
            format!(
                "Vendor rejects {} and holds at {}.",
                crate::ui::format_money(offer),
                crate::ui::format_money(counter)
            ),
        );
        Some(PostAuctionTestResult::Rejected(counter))
    }
}

fn vendor_acceptance_floor(auction: &Auction) -> i64 {
    let concession = match vendor_stance(auction) {
        VendorStance::Flexible => 30_000,
        VendorStance::Negotiable => 20_000,
        VendorStance::Firm => 10_000,
    };
    auction.reserve_price - concession
}
