use crate::app::{App, PurchaseDebrief};
use crate::model::{Auction, AuctionStatus, AuctionTemperature, BidderActor};
use crate::screens::auction_widgets::{
    bid_verdict, draw_heat_bar, guidance_color, money_color, mood_color,
};
use crate::screens::Screen;
use crate::sim::auction_sim::{
    hold_player_position, place_player_bid, place_player_jump_bid, quick_resolve_auction,
    stop_player_bidding, AUCTION_DURATION_SECONDS,
};
use crate::sim::finance::{finance_snapshot, FinanceSnapshot};
use crate::sim::research::estimate_reserve;
use crate::sim::valuation::{cash_needed_to_settle, projected_purchase_margin};
use crate::ui::*;
use macroquad::prelude::*;

enum AuctionUiAction {
    Bid,
    JumpBid,
    Hold,
    WalkAway,
    QuickResolve,
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
        let jump_bid = auction.jump_bid();
        let next_cash_needed = cash_needed_to_settle(next_bid);
        let next_cash_after = self.player.cash - next_cash_needed;
        let next_margin = projected_purchase_margin(&property, next_bid, self.market());
        let finance = finance_snapshot(&self.player, self.market(), next_bid);
        let jump_finance = finance_snapshot(&self.player, self.market(), jump_bid);
        let jump_margin = projected_purchase_margin(&property, jump_bid, self.market());
        let reserve_estimate = estimate_reserve(
            &property,
            self.market(),
            auction.player_research_level,
            self.player.reputation,
        );
        let can_afford_next = finance.can_buy;
        let mut action = None;

        let panel_h = ui_height() - 142.0;
        let left = Rect::new(28.0, 92.0, 302.0, panel_h);
        let center = Rect::new(352.0, 92.0, 520.0, panel_h);
        let right = Rect::new(894.0, 92.0, ui_width() - 922.0, panel_h);

        draw_auction_property_panel(
            left,
            &auction,
            reserve_estimate,
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
                jump_bid,
                jump_margin,
                jump_finance,
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
            Some(AuctionUiAction::JumpBid) => {
                if let Some(auction) = self.current_auction.as_mut() {
                    self.status = place_player_jump_bid(auction);
                }
            }
            Some(AuctionUiAction::Hold) => {
                if let Some(auction) = self.current_auction.as_mut() {
                    let read = hold_player_position(auction);
                    self.status = format!("Held position. {read}");
                }
            }
            Some(AuctionUiAction::WalkAway) => {
                if let Some(auction) = self.current_auction.as_mut() {
                    stop_player_bidding(auction);
                }
            }
            Some(AuctionUiAction::QuickResolve) => {
                if let Some(auction) = self.current_auction.as_mut() {
                    quick_resolve_auction(auction);
                }
            }
            Some(AuctionUiAction::Settle) => self.settle_purchase(),
            Some(AuctionUiAction::ReturnToListings) => {
                if let Some(auction) = &self.current_auction {
                    if matches!(auction.status, Some(AuctionStatus::SoldToNpc(_)))
                        && auction.player_exit_bid.is_some()
                        && auction.current_bid > auction.player_walkaway_price
                    {
                        self.player.reputation += 1;
                        self.status =
                            "Discipline reputation +1 for letting an overheated room win."
                                .to_string();
                    }
                }
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
    reserve_estimate: i64,
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
        ("Reserve estimate", reserve_estimate),
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
        auction.temperature.label(),
        pressure as i32,
        Rect::new(rect.x + 16.0, rect.y + rect.h - 58.0, rect.w - 32.0, 12.0),
        temperature_color(auction.temperature),
    );
    label(
        auction.temperature.description(),
        rect.x + 16.0,
        rect.y + rect.h - 18.0,
        14,
        TEXT_DIM,
    );
    if auction.on_market_announced {
        draw_badge(
            "ON MARKET",
            Rect::new(rect.x + rect.w - 214.0, rect.y + 18.0, 100.0, 28.0),
            POSITIVE,
        );
    }
}

fn draw_live_decision_panel(
    rect: Rect,
    auction: &Auction,
    next_bid: i64,
    margin: i64,
    cash_after: i64,
    can_afford_next: bool,
    finance: FinanceSnapshot,
    jump_bid: i64,
    jump_margin: i64,
    jump_finance: FinanceSnapshot,
) -> Option<AuctionUiAction> {
    soft_panel(rect);
    let over_plan = next_bid > auction.player_walkaway_price;
    let state_color = if over_plan {
        NEGATIVE
    } else {
        temperature_color(auction.temperature)
    };
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, state_color);
    label(
        if over_plan {
            "Over Plan"
        } else {
            auction.temperature.label()
        },
        rect.x + 26.0,
        rect.y + 38.0,
        22,
        state_color,
    );
    label(
        auction.temperature.description(),
        rect.x + 28.0,
        rect.y + 66.0,
        14,
        TEXT_DIM,
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

    if button(
        Rect::new(rect.x + 38.0, rect.y + 330.0, 208.0, 66.0),
        &format!("RAISE {}", format_money(next_bid)),
        auction.is_player_active && can_afford_next,
        if over_plan || !finance.can_buy {
            ButtonTone::Danger
        } else {
            ButtonTone::Primary
        },
    ) {
        return Some(AuctionUiAction::Bid);
    }
    let jump_over_plan = jump_bid > auction.player_walkaway_price;
    let jump_label = if auction.jump_bid_available {
        format!("ASSERT {}", format_money(jump_bid))
    } else {
        "ASSERT USED".to_string()
    };
    if button(
        Rect::new(rect.x + rect.w - 246.0, rect.y + 330.0, 208.0, 66.0),
        &jump_label,
        auction.is_player_active && auction.jump_bid_available && jump_finance.can_buy,
        if jump_over_plan {
            ButtonTone::Danger
        } else {
            ButtonTone::Secondary
        },
    ) {
        return Some(AuctionUiAction::JumpBid);
    }
    label(
        "Raise one step; reveal little.",
        rect.x + 45.0,
        rect.y + 418.0,
        15,
        guidance_color(margin, cash_after, next_bid, auction.player_walkaway_price),
    );
    let jump_note = if auction.jump_bid_available {
        format!("Jump two steps; margin {}.", format_money(jump_margin))
    } else {
        "One assertive jump per auction.".to_string()
    };
    label_fit(
        &jump_note,
        rect.x + rect.w - 238.0,
        rect.y + 418.0,
        200.0,
        15,
        if jump_over_plan { NEGATIVE } else { WARNING },
    );
    draw_wrapped_text(
        auction
            .last_room_read
            .as_deref()
            .unwrap_or("Tap WAIT & READ ROOM to observe a current tell."),
        rect.x + 54.0,
        rect.y + 444.0,
        rect.w - 108.0,
        14,
        TEXT_DIM,
    );

    if !auction.is_player_active {
        label(
            "You are out. Let the room finish without waiting.",
            rect.x + 54.0,
            rect.y + 474.0,
            17,
            TEXT_DIM,
        );
        if button(
            Rect::new(rect.x + 38.0, rect.y + 498.0, rect.w - 76.0, 48.0),
            "Quick Resolve",
            true,
            ButtonTone::Primary,
        ) {
            return Some(AuctionUiAction::QuickResolve);
        }
        return None;
    }

    if button(
        Rect::new(rect.x + 38.0, rect.y + 468.0, 205.0, 48.0),
        "Wait & Read Room",
        auction.is_player_active,
        ButtonTone::Secondary,
    ) {
        return Some(AuctionUiAction::Hold);
    }
    if button(
        Rect::new(rect.x + rect.w - 243.0, rect.y + 468.0, 205.0, 48.0),
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
        let y = rect.y + 76.0 + index as f32 * 82.0;
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
            &format!("{} | {}", bidder.bidder_type.label(), bidder.rhythm),
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
            &format!("Tell: {}", bidder.tell),
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

fn temperature_color(temperature: AuctionTemperature) -> Color {
    match temperature {
        AuctionTemperature::QuietRoom => POSITIVE,
        AuctionTemperature::SteadyInterest => crate::ui::BLUE,
        AuctionTemperature::HeatingUp => WARNING,
        AuctionTemperature::FomoSpiral | AuctionTemperature::FinalCall => NEGATIVE,
    }
}
