use crate::model::Auction;
use crate::sim::finance::FinanceSnapshot;
use crate::sim::valuation::{cash_needed_to_settle, deposit};
use crate::ui::*;
use macroquad::prelude::*;

pub(super) enum AuctionLobbyAction {
    Begin,
    Leave,
}

pub(super) fn draw_auction_day_lobby(
    rect: Rect,
    auction: &Auction,
    finance_at_cap: FinanceSnapshot,
) -> Option<AuctionLobbyAction> {
    soft_panel(rect);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, ACCENT);
    draw_badge(
        "TERMS READ",
        Rect::new(rect.x + rect.w - 126.0, rect.y + 18.0, 98.0, 28.0),
        POSITIVE,
    );
    label("Auction Day", rect.x + 28.0, rect.y + 48.0, 32, TEXT_BRIGHT);
    draw_wrapped_text(
        "Registration is complete. When the hammer falls the contract is unconditional. Check the cash and ceiling before the auctioneer opens the bidding.",
        rect.x + 28.0,
        rect.y + 82.0,
        rect.w - 56.0,
        18,
        TEXT,
    );

    let costs_at_cap = cash_needed_to_settle(auction.player_walkaway_price);
    let rows = [
        ("Safe cash buffer", finance_at_cap.cash_buffer_target),
        ("Your walk-away", auction.player_walkaway_price),
        ("10% deposit at cap", deposit(auction.player_walkaway_price)),
        ("Cash to settle at cap", costs_at_cap),
        ("Cash left at cap", finance_at_cap.cash_after_settle),
        ("Bank room at cap", finance_at_cap.headroom_after),
    ];
    let ledger = Rect::new(rect.x + 26.0, rect.y + 178.0, rect.w - 52.0, 224.0);
    dark_panel(ledger);
    label(
        "BIDDER TERMS",
        ledger.x + 18.0,
        ledger.y + 28.0,
        15,
        TEXT_DIM,
    );
    for (index, (title, value)) in rows.iter().enumerate() {
        draw_value(
            title,
            &format_money(*value),
            ledger.x + 18.0,
            ledger.y + 58.0 + index as f32 * 27.0,
            ledger.w - 36.0,
        );
    }
    label(
        &format!(
            "{} research | {} plan",
            auction.player_research_level.label(),
            auction.walkaway_style.label()
        ),
        rect.x + 28.0,
        rect.y + 430.0,
        17,
        if finance_at_cap.can_buy {
            POSITIVE
        } else {
            NEGATIVE
        },
    );
    label(
        if finance_at_cap.can_buy {
            "Tap START AUCTION CALLS when you are ready. The clock begins then."
        } else {
            "The bank will stop bids before your cap. Lower it next time or leave now."
        },
        rect.x + 28.0,
        rect.y + 458.0,
        15,
        TEXT_DIM,
    );
    if button(
        Rect::new(rect.x + 28.0, rect.y + rect.h - 68.0, 170.0, 48.0),
        "LEAVE AUCTION",
        true,
        ButtonTone::Ghost,
    ) {
        return Some(AuctionLobbyAction::Leave);
    }
    if button(
        Rect::new(rect.x + rect.w - 278.0, rect.y + rect.h - 68.0, 250.0, 48.0),
        "START AUCTION CALLS",
        true,
        ButtonTone::Primary,
    ) {
        return Some(AuctionLobbyAction::Begin);
    }
    None
}
