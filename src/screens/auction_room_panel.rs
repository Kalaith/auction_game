use crate::model::{Auction, BidderActor, RivalRecord};
use crate::screens::auction_widgets::{draw_heat_bar, mood_color};
use crate::ui::*;
use macroquad::prelude::*;

pub(super) fn current_bid_caption(auction: &Auction) -> String {
    match auction.last_bidder.as_ref() {
        Some(BidderActor::Player) => {
            format!(
                "Current Bid · Paddle {} leads",
                auction.player_paddle_number()
            )
        }
        Some(BidderActor::Npc(index)) => format!(
            "Current Bid · {} leads",
            auction
                .bidders
                .get(*index)
                .map(|bidder| bidder.name.as_str())
                .unwrap_or("another bidder")
        ),
        Some(BidderActor::Vendor) => "Declared Vendor Bid · not yet selling".to_string(),
        None => "Opening Call · no leading bidder".to_string(),
    }
}

pub(super) fn draw_bidder_panel(rect: Rect, auction: &Auction, notebook: &[RivalRecord]) {
    soft_panel(rect);
    let active = auction
        .bidders
        .iter()
        .filter(|bidder| bidder.active)
        .count();
    label(
        &format!("Active Bidders: {active}"),
        rect.x + 18.0,
        rect.y + 34.0,
        23,
        TEXT_BRIGHT,
    );
    for (index, bidder) in auction.bidders.iter().enumerate() {
        let y = rect.y + 76.0 + index as f32 * 82.0;
        let history = notebook.iter().find(|record| record.name == bidder.name);
        label_fit(
            &bidder.name,
            rect.x + 18.0,
            y,
            rect.w - 126.0,
            18,
            TEXT_BRIGHT,
        );
        let is_leading = auction.last_bidder == Some(BidderActor::Npc(index));
        label_fit(
            &match history {
                Some(record) => format!(
                    "{} | MET {} | WON {}",
                    bidder.bidder_type.label(),
                    record.auctions_met,
                    record.auctions_won
                ),
                None => format!("{} | FIRST MEETING", bidder.bidder_type.label()),
            },
            rect.x + 18.0,
            y + 22.0,
            rect.w - 36.0,
            15,
            TEXT_DIM,
        );
        label(
            if is_leading {
                "Leading"
            } else {
                bidder.mood.label()
            },
            rect.x + rect.w - 92.0,
            y,
            15,
            if is_leading {
                POSITIVE
            } else {
                mood_color(bidder.mood)
            },
        );
        draw_heat_bar(
            Rect::new(rect.x + 18.0, y + 57.0, rect.w - 36.0, 7.0),
            bidder.heat,
        );
        label_fit(
            &match history {
                Some(record) => format!(
                    "HIGH {} | {}",
                    format_compact_money(record.highest_room_price),
                    bidder.tell
                ),
                None => format!("Tell: {}", bidder.tell),
            },
            rect.x + 18.0,
            y + 44.0,
            rect.w - 36.0,
            13,
            mood_color(bidder.mood),
        );
    }

    label("Auctioneer", rect.x + 18.0, rect.y + 330.0, 22, TEXT_BRIGHT);
    for (index, entry) in auction.log.iter().rev().take(4).enumerate() {
        let text = format!(
            "{:>2}s  {}",
            entry.seconds_remaining.ceil() as i32,
            short_log_line(&entry.text)
        );
        label_fit(
            &text,
            rect.x + 18.0,
            rect.y + 366.0 + index as f32 * 28.0,
            rect.w - 36.0,
            15,
            TEXT_DIM,
        );
    }
}

fn short_log_line(text: &str) -> &str {
    if text.len() > 34 {
        &text[..34]
    } else {
        text
    }
}
