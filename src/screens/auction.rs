use crate::app::{App, PurchaseDebrief};
use crate::model::{Auction, AuctionStatus, BidderActor};
use crate::screens::auction_widgets::{
    bid_verdict, draw_heat_bar, guidance_color, money_color, mood_color,
};
use crate::screens::Screen;
use crate::sim::auction_sim::{place_player_bid, stop_player_bidding, AUCTION_DURATION_SECONDS};
use crate::sim::finance::{finance_snapshot, FinanceSnapshot};
use crate::sim::valuation::{cash_needed_to_settle, projected_purchase_margin};
use crate::ui::*;
use macroquad::prelude::*;

enum AuctionUiAction {
    Bid,
    Hold,
    WalkAway,
    Settle,
    ReturnToListings,
}

impl App {
    pub(crate) fn draw_auction(&mut self) {
        let Some(auction) = self.current_auction.as_ref() else {
            self.screen = Screen::PropertyList;
            return;
        };

        let auction = auction.clone();
        let property = auction.property.clone();
        let next_bid = auction.next_bid();
        let next_cash_needed = cash_needed_to_settle(next_bid);
        let next_cash_after = self.player.cash - next_cash_needed;
        let next_margin = projected_purchase_margin(&property, next_bid, self.market());
        let finance = finance_snapshot(&self.player, self.market(), next_bid);
        let can_afford_next = finance.can_buy;
        let mut action = None;

        let panel_h = ui_height() - 142.0;
        let left = Rect::new(28.0, 92.0, 302.0, panel_h);
        let center = Rect::new(352.0, 92.0, 520.0, panel_h);
        let right = Rect::new(894.0, 92.0, ui_width() - 922.0, panel_h);

        draw_auction_property_panel(
            left,
            &auction,
            next_cash_needed,
            finance.headroom_after,
            next_margin,
        );
        if auction.is_running() {
            action = draw_live_decision_panel(
                center,
                &auction,
                next_bid,
                next_margin,
                next_cash_after,
                can_afford_next,
                finance,
            );
        } else if let Some(status) = auction.status.clone() {
            action = self.draw_auction_result(center, &auction, status);
        }
        draw_bidder_panel(right, &auction);

        match action {
            Some(AuctionUiAction::Bid) => {
                if let Some(auction) = self.current_auction.as_mut() {
                    place_player_bid(auction);
                }
            }
            Some(AuctionUiAction::Hold) => {
                self.status = "You hold your line and let the room move first.".to_string();
            }
            Some(AuctionUiAction::WalkAway) => {
                if let Some(auction) = self.current_auction.as_mut() {
                    stop_player_bidding(auction);
                }
            }
            Some(AuctionUiAction::Settle) => self.settle_purchase(),
            Some(AuctionUiAction::ReturnToListings) => {
                self.current_auction = None;
                self.screen = Screen::PropertyList;
            }
            None => {}
        }
    }

    fn draw_auction_result(
        &self,
        rect: Rect,
        auction: &Auction,
        status: AuctionStatus,
    ) -> Option<AuctionUiAction> {
        soft_panel(rect);
        match status {
            AuctionStatus::SoldToPlayer => {
                let debrief = self.purchase_debrief_for_auction(auction);
                self.draw_purchase_debrief(&debrief, rect);
                if button(
                    Rect::new(rect.x + 36.0, rect.y + rect.h - 64.0, rect.w - 72.0, 44.0),
                    "Settle Purchase",
                    true,
                    ButtonTone::Primary,
                ) {
                    return Some(AuctionUiAction::Settle);
                }
            }
            AuctionStatus::SoldToNpc(name) => {
                label(
                    "Walk-away Held",
                    rect.x + 26.0,
                    rect.y + 46.0,
                    30,
                    TEXT_BRIGHT,
                );
                draw_wrapped_text(
                    &format!(
                        "{name} bought it for {}. Your cash stayed out of a hotter deal.",
                        format_money(auction.current_bid)
                    ),
                    rect.x + 26.0,
                    rect.y + 92.0,
                    rect.w - 52.0,
                    20,
                    TEXT,
                );
                if button(
                    Rect::new(rect.x + 36.0, rect.y + rect.h - 64.0, rect.w - 72.0, 44.0),
                    "Return To Listings",
                    true,
                    ButtonTone::Secondary,
                ) {
                    return Some(AuctionUiAction::ReturnToListings);
                }
            }
            AuctionStatus::PassedIn => {
                label("Passed In", rect.x + 26.0, rect.y + 46.0, 30, TEXT_BRIGHT);
                draw_wrapped_text(
                    "No one reached reserve. That tells you the room was cooler than the quote.",
                    rect.x + 26.0,
                    rect.y + 92.0,
                    rect.w - 52.0,
                    20,
                    TEXT,
                );
                if button(
                    Rect::new(rect.x + 36.0, rect.y + rect.h - 64.0, rect.w - 72.0, 44.0),
                    "Return To Listings",
                    true,
                    ButtonTone::Secondary,
                ) {
                    return Some(AuctionUiAction::ReturnToListings);
                }
            }
        }
        None
    }

    fn draw_purchase_debrief(&self, debrief: &PurchaseDebrief, rect: Rect) {
        label("You Won", rect.x + 26.0, rect.y + 44.0, 32, TEXT_BRIGHT);
        label(&debrief.address, rect.x + 30.0, rect.y + 70.0, 17, TEXT_DIM);
        label(
            &format_money(debrief.purchase_price),
            rect.x + 26.0,
            rect.y + 118.0,
            44,
            ACCENT,
        );
        let values = [
            ("Estimated resale", debrief.estimated_resale),
            ("Cash to settle", debrief.cash_to_settle),
            ("Cash after settle", debrief.cash_after_settle),
            ("Fees", debrief.fees),
            ("Repair allowance", debrief.renovation_allowance),
            ("Projected profit", debrief.projected_profit),
        ];
        for (index, (title, value)) in values.iter().enumerate() {
            draw_value(
                title,
                &format_money(*value),
                rect.x + 28.0,
                rect.y + 160.0 + index as f32 * 27.0,
                rect.w - 56.0,
            );
        }
        if debrief.walkaway_delta > 0 {
            label(
                &format!("Over walk-away by {}", format_money(debrief.walkaway_delta)),
                rect.x + 28.0,
                rect.y + 330.0,
                15,
                NEGATIVE,
            );
        }
        let lesson = Rect::new(rect.x + 24.0, rect.y + 346.0, rect.w - 48.0, 86.0);
        dark_panel(lesson);
        label("Lesson", lesson.x + 14.0, lesson.y + 28.0, 20, TEXT_BRIGHT);
        draw_wrapped_text(
            &debrief.lesson,
            lesson.x + 14.0,
            lesson.y + 56.0,
            lesson.w - 28.0,
            16,
            TEXT,
        );
    }
}

fn draw_auction_property_panel(
    rect: Rect,
    auction: &Auction,
    cash: i64,
    bank_room: i64,
    margin: i64,
) {
    soft_panel(rect);
    draw_house_art(
        Rect::new(rect.x + 14.0, rect.y + 14.0, rect.w - 28.0, 154.0),
        &auction.property,
    );
    label(
        &auction.property.address,
        rect.x + 16.0,
        rect.y + 204.0,
        22,
        TEXT_BRIGHT,
    );
    label(
        &auction.property.suburb,
        rect.x + 16.0,
        rect.y + 230.0,
        17,
        TEXT_DIM,
    );
    let rows = [
        ("Reserve", auction.reserve_price),
        ("Walk-away", auction.player_walkaway_price),
        ("Cash to settle", cash),
        ("Bank room", bank_room),
        ("Margin after fees", margin),
    ];
    for (index, (title, value)) in rows.iter().enumerate() {
        draw_value(
            title,
            &format_money(*value),
            rect.x + 16.0,
            rect.y + 276.0 + index as f32 * 34.0,
            rect.w - 32.0,
        );
    }
    let pressure = 100.0 - auction.seconds_remaining / AUCTION_DURATION_SECONDS * 100.0;
    draw_meter(
        pressure_label(pressure),
        pressure as i32,
        Rect::new(rect.x + 16.0, rect.y + rect.h - 58.0, rect.w - 32.0, 12.0),
        pressure_color(pressure),
    );
    label(
        "Patience protects margin.",
        rect.x + 16.0,
        rect.y + rect.h - 18.0,
        15,
        TEXT_DIM,
    );
}

fn draw_live_decision_panel(
    rect: Rect,
    auction: &Auction,
    next_bid: i64,
    margin: i64,
    cash_after: i64,
    can_afford_next: bool,
    finance: FinanceSnapshot,
) -> Option<AuctionUiAction> {
    soft_panel(rect);
    let pressure = 100.0 - auction.seconds_remaining / AUCTION_DURATION_SECONDS * 100.0;
    let over_plan = next_bid > auction.player_walkaway_price;
    let state_color = if over_plan {
        NEGATIVE
    } else {
        pressure_color(pressure)
    };
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, state_color);
    label(
        if over_plan {
            "Over Plan"
        } else {
            pressure_label(pressure)
        },
        rect.x + 26.0,
        rect.y + 38.0,
        22,
        state_color,
    );
    label(
        &format!("{:02}s", auction.seconds_remaining.ceil() as i32),
        rect.x + rect.w - 104.0,
        rect.y + 42.0,
        34,
        state_color,
    );
    draw_centered_label(
        "Current Bid",
        Rect::new(rect.x + 36.0, rect.y + 86.0, rect.w - 72.0, 28.0),
        19,
        TEXT_DIM,
    );
    draw_centered_label(
        &format_money(auction.current_bid),
        Rect::new(rect.x + 20.0, rect.y + 112.0, rect.w - 40.0, 88.0),
        74,
        ACCENT,
    );
    let verdict = if finance.can_buy {
        bid_verdict(margin, cash_after, next_bid, auction.player_walkaway_price)
    } else {
        finance.stress.label()
    };
    let verdict_color = if finance.can_buy {
        guidance_color(margin, cash_after, next_bid, auction.player_walkaway_price)
    } else {
        NEGATIVE
    };
    draw_badge(
        verdict,
        Rect::new(rect.x + rect.w * 0.5 - 58.0, rect.y + 205.0, 116.0, 28.0),
        verdict_color,
    );
    label("Next Bid", rect.x + 38.0, rect.y + 270.0, 18, TEXT_DIM);
    label(
        &format_money(next_bid),
        rect.x + 38.0,
        rect.y + 304.0,
        31,
        if over_plan { NEGATIVE } else { TEXT_BRIGHT },
    );
    label(
        "Margin after fees",
        rect.x + 300.0,
        rect.y + 270.0,
        18,
        TEXT_DIM,
    );
    label(
        &format_money(margin),
        rect.x + 300.0,
        rect.y + 304.0,
        31,
        money_color(margin),
    );

    let bid_note = if !finance.can_buy {
        "Bank headroom is gone"
    } else if over_plan {
        "Breaks your walk-away price"
    } else {
        "Still within your plan"
    };
    if button(
        Rect::new(rect.x + 38.0, rect.y + 330.0, rect.w - 76.0, 66.0),
        &format!("{verdict} {}", format_money(next_bid)),
        auction.is_player_active && can_afford_next,
        if over_plan || !finance.can_buy {
            ButtonTone::Danger
        } else {
            ButtonTone::Primary
        },
    ) {
        return Some(AuctionUiAction::Bid);
    }
    label(
        bid_note,
        rect.x + 54.0,
        rect.y + 420.0,
        17,
        guidance_color(margin, cash_after, next_bid, auction.player_walkaway_price),
    );

    if button(
        Rect::new(rect.x + 38.0, rect.y + 442.0, 205.0, 48.0),
        "Hold Position",
        auction.is_player_active,
        ButtonTone::Secondary,
    ) {
        return Some(AuctionUiAction::Hold);
    }
    if button(
        Rect::new(rect.x + rect.w - 243.0, rect.y + 442.0, 205.0, 48.0),
        "Walk Away",
        auction.is_player_active,
        ButtonTone::Ghost,
    ) {
        return Some(AuctionUiAction::WalkAway);
    }
    None
}

fn draw_bidder_panel(rect: Rect, auction: &Auction) {
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
        let y = rect.y + 76.0 + index as f32 * 66.0;
        label(
            short_bidder_name(&bidder.name),
            rect.x + 18.0,
            y,
            18,
            TEXT_BRIGHT,
        );
        label(
            bidder.bidder_type.label(),
            rect.x + 18.0,
            y + 22.0,
            16,
            TEXT_DIM,
        );
        let is_leading = auction.last_bidder == Some(BidderActor::Npc(index));
        label(
            if is_leading {
                "Leading"
            } else {
                bidder.mood.label()
            },
            rect.x + 126.0,
            y + 22.0,
            16,
            if is_leading {
                POSITIVE
            } else {
                mood_color(bidder.mood)
            },
        );
        draw_heat_bar(
            Rect::new(rect.x + rect.w - 104.0, y - 7.0, 76.0, 8.0),
            bidder.heat,
        );
    }

    label("Bid Log", rect.x + 18.0, rect.y + 298.0, 22, TEXT_BRIGHT);
    for (index, entry) in auction.log.iter().rev().take(4).enumerate() {
        let text = format!(
            "{:>2}s  {}",
            entry.seconds_remaining.ceil() as i32,
            short_log_line(&entry.text)
        );
        label_fit(
            &text,
            rect.x + 18.0,
            rect.y + 334.0 + index as f32 * 28.0,
            rect.w - 36.0,
            15,
            TEXT_DIM,
        );
    }
}

fn short_bidder_name(name: &str) -> &str {
    if name == "Kestrel Developments" {
        "Kestrel Develop..."
    } else {
        name
    }
}

fn short_log_line(text: &str) -> &str {
    if text.len() > 34 {
        &text[..34]
    } else {
        text
    }
}

fn pressure_label(pressure: f32) -> &'static str {
    if pressure >= 88.0 {
        "Final Call"
    } else if pressure >= 60.0 {
        "Heating Up"
    } else {
        "Calm"
    }
}

fn pressure_color(pressure: f32) -> Color {
    if pressure >= 88.0 {
        NEGATIVE
    } else if pressure >= 60.0 {
        WARNING
    } else {
        POSITIVE
    }
}
