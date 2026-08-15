use crate::app::App;
use crate::model::{Auction, AuctionStatus};
use crate::screens::auction_debrief::draw_purchase_debrief;
use crate::screens::auction_lobby::{draw_auction_day_lobby, AuctionLobbyAction};
use crate::screens::auction_property_panel::draw_auction_property_panel;
use crate::screens::auction_room_panel::{current_bid_caption, draw_bidder_panel};
use crate::screens::auction_widgets::{
    bid_verdict, guidance_color, money_color, temperature_color,
};
use crate::screens::Screen;
use crate::sim::auction_events::{
    accept_post_auction_offer, post_auction_offer, test_vendor_at_passed_in_price, vendor_stance,
    PostAuctionTestResult,
};
use crate::sim::auction_sim::{
    begin_auction_calls, hold_player_position, place_player_bid, place_player_jump_bid,
    quick_resolve_auction, stop_player_bidding,
};
use crate::sim::finance::{finance_snapshot, rental_underwrite, FinanceSnapshot};
use crate::sim::research::estimate_reserve;
use crate::sim::rival_notebook::record_completed_room;
use crate::sim::valuation::{cash_needed_to_settle, projected_purchase_margin};
use crate::ui::*;
use macroquad::prelude::*;

enum AuctionUiAction {
    BeginAuction,
    Bid,
    JumpBid,
    Hold,
    WalkAway,
    QuickResolve,
    Settle,
    ReturnToListings,
    AcceptPostAuction,
    TestPostAuction,
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
        let panel_price = if auction.is_running() {
            next_bid
        } else if auction.status == Some(AuctionStatus::PassedIn) {
            post_auction_offer(&auction).unwrap_or(auction.current_bid)
        } else {
            auction.current_bid
        };
        let panel_finance = finance_snapshot(&self.player, self.market(), panel_price);
        let panel_margin = projected_purchase_margin(&property, panel_price, self.market());
        let panel_rental = rental_underwrite(&property, self.market(), panel_price);
        let mut action = None;

        let panel_h = ui_height() - 142.0;
        let left = Rect::new(28.0, 92.0, 302.0, panel_h);
        let center = Rect::new(352.0, 92.0, 520.0, panel_h);
        let right = Rect::new(894.0, 92.0, ui_width() - 922.0, panel_h);

        draw_auction_property_panel(
            left,
            &auction,
            reserve_estimate,
            cash_needed_to_settle(panel_price),
            panel_finance.headroom_after,
            panel_margin,
            panel_rental.net_cashflow,
        );
        if auction.is_running() && !auction.has_started {
            action = draw_auction_day_lobby(
                center,
                &auction,
                finance_snapshot(&self.player, self.market(), auction.player_walkaway_price),
            )
            .map(|lobby_action| match lobby_action {
                AuctionLobbyAction::Begin => AuctionUiAction::BeginAuction,
                AuctionLobbyAction::Leave => AuctionUiAction::ReturnToListings,
            });
        } else if auction.is_running() {
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
        draw_bidder_panel(right, &auction, &self.player.rival_notebook);

        match action {
            Some(AuctionUiAction::BeginAuction) => {
                if let Some(auction) = self.current_auction.as_mut() {
                    begin_auction_calls(auction);
                    self.status =
                        "Bidding is live. Tap RAISE, ASSERT, WAIT & READ ROOM, or WALK AWAY."
                            .to_string();
                }
            }
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
            Some(AuctionUiAction::Settle) => {
                let homes_before = self.player.properties.len();
                self.settle_purchase();
                if self.player.properties.len() > homes_before {
                    record_completed_room(&mut self.player.rival_notebook, &auction);
                }
            }
            Some(AuctionUiAction::ReturnToListings) => {
                record_completed_room(&mut self.player.rival_notebook, &auction);
                if let Some(auction) = &self.current_auction {
                    if matches!(auction.status, Some(AuctionStatus::SoldToNpc(_)))
                        && auction.player_exit_bid.is_some()
                        && auction.current_bid > auction.player_walkaway_price
                    {
                        self.player.reputation += 1;
                        self.player.career.disciplined_walkaways += 1;
                        self.status =
                            "Discipline reputation +1 for letting an overheated room win."
                                .to_string();
                    }
                }
                self.current_auction = None;
                self.screen = Screen::PropertyList;
            }
            Some(AuctionUiAction::AcceptPostAuction) => {
                if let Some(auction) = self.current_auction.as_mut() {
                    if accept_post_auction_offer(auction) {
                        self.status =
                            "Vendor accepted. Review the numbers, then tap SETTLE PURCHASE."
                                .to_string();
                    }
                }
            }
            Some(AuctionUiAction::TestPostAuction) => {
                if let Some(auction) = self.current_auction.as_mut() {
                    match test_vendor_at_passed_in_price(auction) {
                        Some(PostAuctionTestResult::Accepted(price)) => {
                            self.status = format!(
                                "Vendor accepted your {} offer. Review, then tap SETTLE PURCHASE.",
                                format_money(price)
                            );
                        }
                        Some(PostAuctionTestResult::Rejected(counter)) => {
                            self.status = format!(
                                "Vendor rejected the test offer and holds at {}.",
                                format_money(counter)
                            );
                        }
                        None => {}
                    }
                }
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
                draw_purchase_debrief(&debrief, rect);
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
                let walked_away = auction.player_exit_bid.is_some();
                label(
                    if walked_away {
                        "Walk-away Held"
                    } else {
                        "Outbid At The Hammer"
                    },
                    rect.x + 26.0,
                    rect.y + 46.0,
                    30,
                    TEXT_BRIGHT,
                );
                draw_wrapped_text(
                    &if walked_away {
                        format!(
                            "{name} bought it for {}. Your cash stayed out of a hotter deal.",
                            format_money(auction.current_bid)
                        )
                    } else {
                        format!(
                            "{name} held the final bid at {}. You kept the paddle down when the last call came.",
                            format_money(auction.current_bid)
                        )
                    },
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
                label(
                    "Passed In — Negotiation",
                    rect.x + 26.0,
                    rect.y + 46.0,
                    30,
                    TEXT_BRIGHT,
                );
                draw_wrapped_text(
                    "The public auction missed reserve. The vendor's agent will now name a private counteroffer; you may still leave.",
                    rect.x + 26.0,
                    rect.y + 92.0,
                    rect.w - 52.0,
                    20,
                    TEXT,
                );
                let offer = post_auction_offer(auction).unwrap_or(auction.reserve_price);
                let offer_finance = finance_snapshot(&self.player, self.market(), offer);
                let test_finance =
                    finance_snapshot(&self.player, self.market(), auction.current_bid);
                label(
                    "Vendor counteroffer",
                    rect.x + 26.0,
                    rect.y + 166.0,
                    17,
                    TEXT_DIM,
                );
                label(
                    &format_money(offer),
                    rect.x + 26.0,
                    rect.y + 208.0,
                    42,
                    if offer <= auction.player_walkaway_price {
                        POSITIVE
                    } else {
                        WARNING
                    },
                );
                let walkaway_relation = if offer <= auction.player_walkaway_price {
                    "below"
                } else {
                    "above"
                };
                label(
                    &format!(
                        "{} {walkaway_relation} walk-away | cash after settle {}",
                        format_money((offer - auction.player_walkaway_price).abs()),
                        format_money(offer_finance.cash_after_settle)
                    ),
                    rect.x + 28.0,
                    rect.y + 240.0,
                    16,
                    TEXT_DIM,
                );
                label_fit(
                    if auction.post_auction_tested {
                        "Your lower offer was rejected. The vendor counter still stands."
                    } else {
                        "Test the vendor at the passed-in price, or meet their counter now."
                    },
                    rect.x + 28.0,
                    rect.y + 270.0,
                    rect.w - 56.0,
                    15,
                    if auction.post_auction_tested {
                        WARNING
                    } else {
                        crate::ui::BLUE
                    },
                );
                label_fit(
                    if auction.player_research_level >= crate::model::ResearchLevel::FullDiligence {
                        vendor_stance(auction).label()
                    } else {
                        "Seller flexibility is unclear without full diligence."
                    },
                    rect.x + 28.0,
                    rect.y + 300.0,
                    rect.w - 56.0,
                    15,
                    if auction.player_research_level >= crate::model::ResearchLevel::FullDiligence {
                        POSITIVE
                    } else {
                        TEXT_DIM
                    },
                );
                if button(
                    Rect::new(rect.x + 26.0, rect.y + rect.h - 64.0, 148.0, 44.0),
                    &if auction.post_auction_tested {
                        "OFFER REJECTED".to_string()
                    } else {
                        format!("OFFER {}", format_money(auction.current_bid))
                    },
                    !auction.post_auction_tested && test_finance.can_buy,
                    ButtonTone::Secondary,
                ) {
                    return Some(AuctionUiAction::TestPostAuction);
                }
                if button(
                    Rect::new(rect.x + 184.0, rect.y + rect.h - 64.0, 166.0, 44.0),
                    &format!("MEET {}", format_money(offer)),
                    offer_finance.can_buy,
                    if offer <= auction.player_walkaway_price {
                        ButtonTone::Primary
                    } else {
                        ButtonTone::Danger
                    },
                ) {
                    return Some(AuctionUiAction::AcceptPostAuction);
                }
                if button(
                    Rect::new(rect.x + rect.w - 150.0, rect.y + rect.h - 64.0, 124.0, 44.0),
                    "LEAVE",
                    true,
                    ButtonTone::Ghost,
                ) {
                    return Some(AuctionUiAction::ReturnToListings);
                }
            }
        }
        None
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
        &current_bid_caption(auction),
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
        bid_verdict(
            margin,
            cash_after,
            finance.cash_buffer_target,
            next_bid,
            auction.player_walkaway_price,
        )
    } else {
        finance.stress.label()
    };
    let verdict_color = if finance.can_buy {
        guidance_color(
            margin,
            cash_after,
            finance.cash_buffer_target,
            next_bid,
            auction.player_walkaway_price,
        )
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
        guidance_color(
            margin,
            cash_after,
            finance.cash_buffer_target,
            next_bid,
            auction.player_walkaway_price,
        ),
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
