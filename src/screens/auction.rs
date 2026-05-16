use crate::app::{App, PurchaseDebrief};
use crate::model::AuctionStatus;
use crate::screens::auction_widgets::{
    bid_guidance, draw_heat_bar, guidance_color, money_color, mood_color,
};
use crate::screens::Screen;
use crate::sim::auction_sim::{place_player_bid, stop_player_bidding, AUCTION_DURATION_SECONDS};
use crate::sim::valuation::{cash_needed_to_settle, projected_purchase_margin};
use crate::ui::*;
use macroquad::prelude::*;

enum AuctionUiAction {
    Bid,
    Stop,
    Settle,
    ReturnToListings,
}

impl App {
    pub(crate) fn draw_auction(&mut self) {
        let Some(auction) = self.current_auction.as_ref() else {
            self.screen = Screen::PropertyList;
            return;
        };

        let property = auction.property.clone();
        let auction_status = auction.status.clone();
        let can_afford_next = cash_needed_to_settle(auction.next_bid()) <= self.player.cash;
        let next_bid = auction.next_bid();
        let next_cash_needed = cash_needed_to_settle(next_bid);
        let next_cash_after = self.player.cash - next_cash_needed;
        let next_margin = projected_purchase_margin(&property, next_bid, self.market());
        let mut action = None;

        let left = Rect::new(28.0, 92.0, 310.0, 540.0);
        panel(left);
        draw_house_art(
            Rect::new(left.x + 14.0, left.y + 14.0, left.w - 28.0, 170.0),
            &property,
        );
        label(
            &property.address,
            left.x + 16.0,
            left.y + 214.0,
            23,
            TEXT_BRIGHT,
        );
        label(
            &property.suburb,
            left.x + 16.0,
            left.y + 242.0,
            18,
            TEXT_DIM,
        );
        draw_value(
            "Reserve",
            &format_money(auction.reserve_price),
            left.x + 16.0,
            left.y + 286.0,
            left.w - 32.0,
        );
        draw_value(
            "Walk-away",
            &format_money(auction.player_walkaway_price),
            left.x + 16.0,
            left.y + 324.0,
            left.w - 32.0,
        );
        draw_value(
            "Cash to settle next",
            &format_money(next_cash_needed),
            left.x + 16.0,
            left.y + 362.0,
            left.w - 32.0,
        );
        draw_value(
            "Cash after next",
            &format_money(next_cash_after),
            left.x + 16.0,
            left.y + 394.0,
            left.w - 32.0,
        );
        draw_value(
            "Margin at next",
            &format_money(next_margin),
            left.x + 16.0,
            left.y + 426.0,
            left.w - 32.0,
        );
        draw_meter(
            "Pressure",
            (100.0 - auction.seconds_remaining / AUCTION_DURATION_SECONDS * 100.0) as i32,
            Rect::new(left.x + 16.0, left.y + 466.0, left.w - 32.0, 14.0),
            WARNING,
        );
        draw_wrapped_text(
            "Use stop bidding as a skill, not a surrender button.",
            left.x + 16.0,
            left.y + 516.0,
            left.w - 32.0,
            16,
            TEXT_DIM,
        );

        let center = Rect::new(360.0, 92.0, 458.0, 540.0);
        panel(center);
        label(
            "Live Auction",
            center.x + 18.0,
            center.y + 36.0,
            28,
            TEXT_BRIGHT,
        );
        label(
            &format!("Current bid {}", format_money(auction.current_bid)),
            center.x + 18.0,
            center.y + 102.0,
            34,
            ACCENT,
        );
        label(
            &format!(
                "{} seconds remaining",
                auction.seconds_remaining.ceil() as i32
            ),
            center.x + 18.0,
            center.y + 140.0,
            22,
            TEXT,
        );
        let leading = match auction.last_bidder {
            Some(crate::model::BidderActor::Player) => "You are leading".to_string(),
            Some(crate::model::BidderActor::Npc(index)) => {
                format!("Leading: {}", auction.bidders[index].name)
            }
            None => "Waiting for an opening bid".to_string(),
        };
        label(&leading, center.x + 18.0, center.y + 174.0, 20, TEXT_DIM);

        if auction.is_running() {
            let guidance = bid_guidance(
                next_margin,
                next_cash_after,
                next_bid,
                auction.player_walkaway_price,
            );
            label(
                &format!("Next bid margin: {}", format_money(next_margin)),
                center.x + 34.0,
                center.y + 210.0,
                19,
                money_color(next_margin),
            );
            let bid_color = if next_bid > auction.player_walkaway_price {
                ButtonTone::Danger
            } else {
                ButtonTone::Primary
            };
            if button(
                Rect::new(center.x + 34.0, center.y + 236.0, center.w - 68.0, 58.0),
                &format!("Bid {}", format_money(next_bid)),
                auction.is_player_active && can_afford_next,
                bid_color,
            ) {
                action = Some(AuctionUiAction::Bid);
            }
            if button(
                Rect::new(center.x + 34.0, center.y + 312.0, center.w - 68.0, 48.0),
                "Stop Bidding",
                auction.is_player_active,
                ButtonTone::Secondary,
            ) {
                action = Some(AuctionUiAction::Stop);
            }
            if !can_afford_next {
                draw_wrapped_text(
                    &format!(
                        "Need {} to settle the next bid. Current cash is {}.",
                        format_money(next_cash_needed),
                        format_money(self.player.cash)
                    ),
                    center.x + 34.0,
                    center.y + 392.0,
                    center.w - 68.0,
                    18,
                    NEGATIVE,
                );
            } else {
                draw_wrapped_text(
                    guidance,
                    center.x + 34.0,
                    center.y + 392.0,
                    center.w - 68.0,
                    18,
                    guidance_color(
                        next_margin,
                        next_cash_after,
                        next_bid,
                        auction.player_walkaway_price,
                    ),
                );
            }
        } else if let Some(status) = auction_status {
            let result_rect = Rect::new(center.x + 28.0, center.y + 174.0, center.w - 56.0, 390.0);
            dark_panel(result_rect);
            match status {
                AuctionStatus::SoldToPlayer => {
                    let debrief = self.purchase_debrief_for_auction(auction);
                    self.draw_purchase_debrief(&debrief, result_rect);
                    if button(
                        Rect::new(
                            result_rect.x + 18.0,
                            result_rect.y + result_rect.h - 54.0,
                            result_rect.w - 36.0,
                            38.0,
                        ),
                        "Settle Purchase",
                        true,
                        ButtonTone::Primary,
                    ) {
                        action = Some(AuctionUiAction::Settle);
                    }
                }
                AuctionStatus::SoldToNpc(name) => {
                    label(
                        "Auction Result",
                        result_rect.x + 18.0,
                        result_rect.y + 34.0,
                        24,
                        TEXT_BRIGHT,
                    );
                    let result_text = if let Some(exit_bid) = auction.player_exit_bid {
                        format!(
                            "You stopped at {}. {name} paid {}, which kept {} of extra risk off your books.",
                            format_money(exit_bid),
                            format_money(auction.current_bid),
                            format_money(auction.current_bid - exit_bid)
                        )
                    } else {
                        format!(
                            "{name} bought it for {}. Walking away kept your cash intact.",
                            format_money(auction.current_bid)
                        )
                    };
                    draw_wrapped_text(
                        &result_text,
                        result_rect.x + 18.0,
                        result_rect.y + 74.0,
                        result_rect.w - 36.0,
                        19,
                        TEXT,
                    );
                    if button(
                        Rect::new(
                            result_rect.x + 18.0,
                            result_rect.y + result_rect.h - 54.0,
                            result_rect.w - 36.0,
                            38.0,
                        ),
                        "Return To Listings",
                        true,
                        ButtonTone::Secondary,
                    ) {
                        action = Some(AuctionUiAction::ReturnToListings);
                    }
                }
                AuctionStatus::PassedIn => {
                    label(
                        "Auction Passed In",
                        result_rect.x + 18.0,
                        result_rect.y + 34.0,
                        24,
                        TEXT_BRIGHT,
                    );
                    let result_text = if let Some(exit_bid) = auction.player_exit_bid {
                        format!(
                            "No one reached reserve after you stopped at {}. That is a useful read on the room.",
                            format_money(exit_bid)
                        )
                    } else {
                        "The reserve was not met. Sometimes the cleanest bid is the one you never make.".to_string()
                    };
                    draw_wrapped_text(
                        &result_text,
                        result_rect.x + 18.0,
                        result_rect.y + 74.0,
                        result_rect.w - 36.0,
                        19,
                        TEXT,
                    );
                    if button(
                        Rect::new(
                            result_rect.x + 18.0,
                            result_rect.y + result_rect.h - 54.0,
                            result_rect.w - 36.0,
                            38.0,
                        ),
                        "Return To Listings",
                        true,
                        ButtonTone::Secondary,
                    ) {
                        action = Some(AuctionUiAction::ReturnToListings);
                    }
                }
            }
        }

        let right = Rect::new(842.0, 92.0, screen_width() - 870.0, 540.0);
        panel(right);
        label("Bidders", right.x + 18.0, right.y + 34.0, 24, TEXT_BRIGHT);
        for (index, bidder) in auction.bidders.iter().enumerate() {
            let y = right.y + 70.0 + index as f32 * 72.0;
            label(&bidder.name, right.x + 18.0, y, 19, TEXT_BRIGHT);
            label(
                &format!("{} | {}", bidder.bidder_type.label(), bidder.mood.label()),
                right.x + 18.0,
                y + 24.0,
                16,
                if bidder.active {
                    mood_color(bidder.mood)
                } else {
                    NEGATIVE
                },
            );
            draw_heat_bar(
                Rect::new(right.x + right.w - 110.0, y + 12.0, 78.0, 8.0),
                bidder.heat,
            );
            draw_wrapped_text(
                &bidder.tell,
                right.x + 18.0,
                y + 46.0,
                right.w - 42.0,
                14,
                TEXT_DIM,
            );
        }

        label("Bid Log", right.x + 18.0, right.y + 300.0, 24, TEXT_BRIGHT);
        for (index, entry) in auction.log.iter().rev().take(6).enumerate() {
            let y = right.y + 338.0 + index as f32 * 30.0;
            label(
                &format!(
                    "{:>2}s  {}",
                    entry.seconds_remaining.ceil() as i32,
                    entry.text
                ),
                right.x + 18.0,
                y,
                16,
                TEXT,
            );
        }

        match action {
            Some(AuctionUiAction::Bid) => {
                if let Some(auction) = self.current_auction.as_mut() {
                    place_player_bid(auction);
                }
            }
            Some(AuctionUiAction::Stop) => {
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

    fn draw_purchase_debrief(&self, debrief: &PurchaseDebrief, rect: Rect) {
        label(
            "Auction Result",
            rect.x + 18.0,
            rect.y + 32.0,
            24,
            TEXT_BRIGHT,
        );
        label(
            &format!(
                "You won {} at {}",
                debrief.address,
                format_money(debrief.purchase_price)
            ),
            rect.x + 18.0,
            rect.y + 64.0,
            18,
            ACCENT,
        );
        draw_value(
            "Estimated resale",
            &format_money(debrief.estimated_resale),
            rect.x + 18.0,
            rect.y + 96.0,
            rect.w - 36.0,
        );
        draw_value(
            "Cash to settle",
            &format_money(debrief.cash_to_settle),
            rect.x + 18.0,
            rect.y + 126.0,
            rect.w - 36.0,
        );
        draw_value(
            "Cash after settle",
            &format_money(debrief.cash_after_settle),
            rect.x + 18.0,
            rect.y + 156.0,
            rect.w - 36.0,
        );
        draw_value(
            "Fees",
            &format_money(debrief.fees),
            rect.x + 18.0,
            rect.y + 186.0,
            rect.w - 36.0,
        );
        draw_value(
            "Repair allowance",
            &format_money(debrief.renovation_allowance),
            rect.x + 18.0,
            rect.y + 216.0,
            rect.w - 36.0,
        );
        draw_value(
            "Projected profit",
            &format_money(debrief.projected_profit),
            rect.x + 18.0,
            rect.y + 246.0,
            rect.w - 36.0,
        );
        if debrief.walkaway_delta > 0 {
            label(
                &format!("Over walk-away by {}", format_money(debrief.walkaway_delta)),
                rect.x + 18.0,
                rect.y + 268.0,
                16,
                NEGATIVE,
            );
        }
        draw_wrapped_text(
            &debrief.lesson,
            rect.x + 18.0,
            rect.y + 286.0,
            rect.w - 36.0,
            16,
            TEXT,
        );
    }
}
