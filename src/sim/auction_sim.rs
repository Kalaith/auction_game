use crate::model::{
    Auction, AuctionStatus, BidLog, Bidder, BidderActor, BidderMood, BidderProfileData, BidderType,
    Condition, MarketEvent, Property,
};
use crate::sim::valuation::{market_adjusted_value, round_down_to_increment};
use macroquad::rand::gen_range;

pub const AUCTION_DURATION_SECONDS: f32 = 56.0;
const BID_INCREMENT: i64 = 10_000;

pub fn create_auction(
    property: &Property,
    market: &MarketEvent,
    profiles: &[BidderProfileData],
    player_walkaway_price: i64,
) -> Auction {
    let mut bidders = Vec::new();
    for offset in 0..3 {
        let profile = &profiles[(property.id + offset) % profiles.len()];
        let max_price = bidder_ceiling(property, market, profile);
        bidders.push(Bidder {
            name: profile.name.clone(),
            bidder_type: profile.bidder_type,
            max_price,
            aggression: profile.aggression,
            patience: profile.patience,
            reaction_timer: 1.0 + offset as f32 * 0.55,
            bid_count: 0,
            heat: 35 + (profile.aggression * 35.0) as i32,
            mood: BidderMood::Watching,
            tell: opening_tell(profile.bidder_type).to_string(),
            active: true,
            has_logged_exit: false,
            stretch_bid_used: false,
        });
    }

    let opening_bid = round_down_to_increment(
        (property.guide_price - 40_000).max(property.guide_price / 2),
        BID_INCREMENT,
    );

    Auction {
        property: property.clone(),
        current_bid: opening_bid,
        reserve_price: property.reserve_price,
        bid_increment: BID_INCREMENT,
        seconds_remaining: AUCTION_DURATION_SECONDS,
        call_timer: 1.0,
        bidders,
        last_bidder: None,
        is_player_active: true,
        player_exit_bid: None,
        overtime_count: 0,
        status: None,
        log: vec![BidLog {
            text: format!("Auction opens at {}.", crate::ui::format_money(opening_bid)),
            seconds_remaining: AUCTION_DURATION_SECONDS,
        }],
        player_walkaway_price,
    }
}

pub fn update_auction(auction: &mut Auction, dt: f32) {
    if !auction.is_running() {
        return;
    }

    auction.seconds_remaining = (auction.seconds_remaining - dt).max(0.0);
    auction.call_timer -= dt;

    if auction.call_timer <= 0.0 {
        let line = auctioneer_line(auction);
        push_log(auction, line);
        auction.call_timer = gen_range(1.6, 3.0);
    }

    let mut bidder_to_place = None;
    for index in 0..auction.bidders.len() {
        if !auction.bidders[index].active {
            continue;
        }
        if auction.last_bidder == Some(BidderActor::Npc(index)) {
            continue;
        }

        auction.bidders[index].reaction_timer -= dt;
        if auction.bidders[index].reaction_timer > 0.0 {
            continue;
        }

        let next_bid = auction.next_bid();
        update_bidder_tell(auction, index, next_bid);
        if next_bid > bidder_effective_ceiling(auction, index) {
            retire_bidder(auction, index);
            continue;
        }

        let chance = bid_chance(auction, index, next_bid);

        if gen_range(0.0, 1.0) < chance {
            bidder_to_place = Some(index);
            break;
        }

        auction.bidders[index].reaction_timer = gen_range(1.1, 3.1);
    }

    if let Some(index) = bidder_to_place {
        place_npc_bid(auction, index);
    }

    if auction.seconds_remaining <= 0.0 {
        finish_auction(auction);
    }
}

pub fn place_player_bid(auction: &mut Auction) {
    if !auction.is_running() || !auction.is_player_active {
        return;
    }

    let next_bid = auction.next_bid();
    auction.current_bid = next_bid;
    auction.last_bidder = Some(BidderActor::Player);
    extend_if_needed(auction);

    if next_bid > auction.player_walkaway_price {
        push_log(
            auction,
            format!(
                "You bid {}. That is above your walk-away reminder.",
                crate::ui::format_money(next_bid)
            ),
        );
    } else {
        push_log(
            auction,
            format!("You bid {}.", crate::ui::format_money(next_bid)),
        );
    }
}

pub fn stop_player_bidding(auction: &mut Auction) {
    if !auction.is_running() || !auction.is_player_active {
        return;
    }
    auction.is_player_active = false;
    auction.player_exit_bid = Some(auction.current_bid);
    push_log(
        auction,
        format!(
            "You stop at {} and force the room to prove it.",
            crate::ui::format_money(auction.current_bid)
        ),
    );
}

fn place_npc_bid(auction: &mut Auction, index: usize) {
    let previous_bid = auction.current_bid;
    let next_bid = npc_bid_amount(auction, index);
    let was_stretch = next_bid > auction.bidders[index].max_price;
    auction.current_bid = next_bid;
    auction.last_bidder = Some(BidderActor::Npc(index));
    auction.bidders[index].bid_count += 1;
    auction.bidders[index].heat = (auction.bidders[index].heat + 12).min(100);
    if was_stretch {
        auction.bidders[index].stretch_bid_used = true;
        auction.bidders[index].mood = BidderMood::Stretching;
    } else {
        auction.bidders[index].mood = BidderMood::Interested;
    }
    auction.bidders[index].tell = bid_tell(&auction.bidders[index]).to_string();
    auction.bidders[index].reaction_timer = gen_range(1.2, 3.2);
    extend_if_needed(auction);

    let name = auction.bidders[index].name.clone();
    let action = if was_stretch {
        "stretches to"
    } else if next_bid - previous_bid > auction.bid_increment {
        "jumps to"
    } else {
        "bids"
    };
    push_log(
        auction,
        format!("{name} {action} {}.", crate::ui::format_money(next_bid)),
    );
}

fn finish_auction(auction: &mut Auction) {
    if auction.current_bid < auction.reserve_price || auction.last_bidder.is_none() {
        auction.status = Some(AuctionStatus::PassedIn);
        push_log(auction, "Auction passes in below reserve.".to_string());
        return;
    }

    match auction.last_bidder {
        Some(BidderActor::Player) => {
            auction.status = Some(AuctionStatus::SoldToPlayer);
            push_log(
                auction,
                format!(
                    "Sold to you for {}.",
                    crate::ui::format_money(auction.current_bid)
                ),
            );
        }
        Some(BidderActor::Npc(index)) => {
            let name = auction.bidders[index].name.clone();
            auction.status = Some(AuctionStatus::SoldToNpc(name.clone()));
            push_log(
                auction,
                format!(
                    "Sold to {name} for {}.",
                    crate::ui::format_money(auction.current_bid)
                ),
            );
        }
        None => {}
    }
}

fn extend_if_needed(auction: &mut Auction) {
    if auction.seconds_remaining < 8.0 {
        auction.overtime_count += 1;
        auction.seconds_remaining = if auction.overtime_count >= 3 {
            7.0
        } else {
            11.0
        };
        push_log(
            auction,
            if auction.overtime_count >= 3 {
                "The auctioneer warns there will be no more patience.".to_string()
            } else {
                "Late bid. The auctioneer gives the room one more chance.".to_string()
            },
        );
    }
}

fn push_log(auction: &mut Auction, text: String) {
    auction.log.push(BidLog {
        text,
        seconds_remaining: auction.seconds_remaining,
    });
    if auction.log.len() > 12 {
        auction.log.remove(0);
    }
}

fn auctioneer_line(auction: &Auction) -> String {
    if auction.seconds_remaining < 8.0 && auction.overtime_count >= 3 {
        "Last warning. The next silence ends it.".to_string()
    } else if auction.seconds_remaining < 8.0 {
        format!(
            "Final call at {}. Any better offer?",
            crate::ui::format_money(auction.current_bid)
        )
    } else if auction.seconds_remaining < 18.0 {
        format!(
            "Pressure rises. The next call is {}.",
            crate::ui::format_money(auction.next_bid())
        )
    } else if auction.current_bid < auction.reserve_price {
        "Still looking for a bid that meets reserve.".to_string()
    } else if auction.last_bidder == Some(BidderActor::Player) {
        "You are holding the top bid. Stay disciplined.".to_string()
    } else {
        format!(
            "Auctioneer asks for {}.",
            crate::ui::format_money(auction.next_bid())
        )
    }
}

fn retire_bidder(auction: &mut Auction, index: usize) {
    auction.bidders[index].active = false;
    auction.bidders[index].mood = BidderMood::Out;
    auction.bidders[index].heat = 0;
    auction.bidders[index].tell = exit_tell(auction.bidders[index].bidder_type).to_string();
    if !auction.bidders[index].has_logged_exit {
        auction.bidders[index].has_logged_exit = true;
        let name = auction.bidders[index].name.clone();
        push_log(auction, format!("{name} folds at the current price."));
    }
}

fn bidder_effective_ceiling(auction: &Auction, index: usize) -> i64 {
    let bidder = &auction.bidders[index];
    if bidder.stretch_bid_used {
        return bidder.max_price;
    }

    let can_stretch = match bidder.bidder_type {
        BidderType::FirstHomeBuyer => auction.seconds_remaining < 16.0,
        BidderType::EgoBidder => auction.last_bidder == Some(BidderActor::Player),
        _ => false,
    };

    if can_stretch {
        bidder.max_price + auction.bid_increment
    } else {
        bidder.max_price
    }
}

fn bid_chance(auction: &Auction, index: usize, next_bid: i64) -> f32 {
    let bidder = &auction.bidders[index];
    let value_room = ((bidder.max_price - auction.current_bid) as f32 / bidder.max_price as f32)
        .clamp(0.0, 0.25);
    let urgency = if auction.seconds_remaining < 10.0 {
        0.34
    } else if auction.seconds_remaining < 22.0 {
        0.16
    } else {
        0.05
    };
    let mut chance = (bidder.aggression * 0.42) + (bidder.patience * 0.10) + value_room + urgency;

    match bidder.bidder_type {
        BidderType::FirstHomeBuyer => {
            chance += if auction.last_bidder == Some(BidderActor::Player) {
                0.13
            } else {
                0.03
            };
        }
        BidderType::Investor => {
            chance -= if next_bid > auction.reserve_price {
                0.09
            } else {
                0.02
            };
        }
        BidderType::Renovator => {
            if matches!(
                auction.property.condition,
                Condition::Rough | Condition::Tired
            ) {
                chance += 0.12;
            }
        }
        BidderType::Developer => {
            if auction.property.land_size >= 600 {
                chance += 0.14;
            } else {
                chance -= 0.08;
            }
        }
        BidderType::EgoBidder => {
            chance += if auction.last_bidder == Some(BidderActor::Player) {
                0.22
            } else {
                0.07
            };
        }
        BidderType::BargainHunter => {
            if next_bid > (auction.property.market_value as f32 * 0.94) as i64 {
                chance -= 0.28;
            }
        }
    }

    chance.clamp(0.03, 0.92)
}

fn npc_bid_amount(auction: &Auction, index: usize) -> i64 {
    let bidder = &auction.bidders[index];
    let mut jump_increments = 1;

    match bidder.bidder_type {
        BidderType::Developer => {
            if auction.property.land_size >= 600 && auction.current_bid < auction.reserve_price {
                jump_increments = 2;
            }
        }
        BidderType::EgoBidder => {
            if auction.last_bidder == Some(BidderActor::Player) && bidder.bid_count > 0 {
                jump_increments = 2;
            }
        }
        _ => {}
    }

    let desired = auction.current_bid + auction.bid_increment * jump_increments;
    desired.min(bidder_effective_ceiling(auction, index))
}

fn update_bidder_tell(auction: &mut Auction, index: usize, next_bid: i64) {
    if !auction.bidders[index].active {
        return;
    }

    let effective_ceiling = bidder_effective_ceiling(auction, index);
    let headroom = effective_ceiling - next_bid;
    let mood = if next_bid > auction.bidders[index].max_price {
        BidderMood::Stretching
    } else if headroom <= auction.bid_increment {
        BidderMood::Hesitating
    } else if auction.last_bidder == Some(BidderActor::Player) {
        BidderMood::Interested
    } else {
        BidderMood::Watching
    };

    auction.bidders[index].mood = mood;
    auction.bidders[index].heat = match mood {
        BidderMood::Watching => 35,
        BidderMood::Interested => 62,
        BidderMood::Hesitating => 44,
        BidderMood::Stretching => 86,
        BidderMood::Out => 0,
    };
    auction.bidders[index].tell =
        tell_for(auction.bidders[index].bidder_type, mood, &auction.property).to_string();
}

fn opening_tell(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "Clutching a pre-approval letter.",
        BidderType::Investor => "Running numbers before every call.",
        BidderType::Renovator => "Inspecting cracks and smiling anyway.",
        BidderType::Developer => "Only watching the land component.",
        BidderType::EgoBidder => "Standing too close to the auctioneer.",
        BidderType::BargainHunter => "Waiting for someone else to blink.",
    }
}

fn exit_tell(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "Partner says no more.",
        BidderType::Investor => "Yield no longer works.",
        BidderType::Renovator => "Repair budget is gone.",
        BidderType::Developer => "Land value ceiling reached.",
        BidderType::EgoBidder => "Pride finally gets expensive.",
        BidderType::BargainHunter => "No longer a bargain.",
    }
}

fn bid_tell(bidder: &Bidder) -> &'static str {
    match bidder.bidder_type {
        BidderType::FirstHomeBuyer => "Bids fast, then looks worried.",
        BidderType::Investor => "Raises only after checking the margin.",
        BidderType::Renovator => "Keeps bidding through cosmetic damage.",
        BidderType::Developer => "Uses big jumps to clear weak bidders.",
        BidderType::EgoBidder => "Answers your bid almost personally.",
        BidderType::BargainHunter => "Bids reluctantly and hates it.",
    }
}

fn tell_for(bidder_type: BidderType, mood: BidderMood, property: &Property) -> &'static str {
    match (bidder_type, mood) {
        (_, BidderMood::Out) => exit_tell(bidder_type),
        (BidderType::FirstHomeBuyer, BidderMood::Hesitating) => "Whispering about their limit.",
        (BidderType::FirstHomeBuyer, BidderMood::Stretching) => {
            "Emotion is beating the spreadsheet."
        }
        (BidderType::Investor, BidderMood::Interested) => "Still under their yield ceiling.",
        (BidderType::Investor, BidderMood::Hesitating) => "Nearly out on the numbers.",
        (BidderType::Renovator, BidderMood::Interested)
            if matches!(property.condition, Condition::Rough | Condition::Tired) =>
        {
            "Seeing upside in the damage."
        }
        (BidderType::Developer, BidderMood::Interested) if property.land_size >= 600 => {
            "Land size keeps them engaged."
        }
        (BidderType::EgoBidder, BidderMood::Interested) => "Watching your paddle, not the house.",
        (BidderType::EgoBidder, BidderMood::Stretching) => "This is becoming personal.",
        (BidderType::BargainHunter, BidderMood::Hesitating) => "Value buffer is nearly gone.",
        (_, BidderMood::Interested) => "Still engaged.",
        (_, BidderMood::Hesitating) => "Close to their ceiling.",
        (_, BidderMood::Stretching) => "Pushing past comfort.",
        (_, BidderMood::Watching) => opening_tell(bidder_type),
    }
}

fn bidder_ceiling(property: &Property, market: &MarketEvent, profile: &BidderProfileData) -> i64 {
    let mut modifier = profile.budget_bias + market.buyer_budget_modifier;

    match profile.bidder_type {
        BidderType::FirstHomeBuyer => {
            modifier += (property.appeal as f32 - 55.0) / 1000.0;
        }
        BidderType::Investor => {
            modifier -= 0.03;
            modifier += (property.buyer_demand as f32 - 55.0) / 1200.0;
        }
        BidderType::Renovator => {
            if matches!(property.condition, Condition::Rough | Condition::Tired) {
                modifier += property.renovation_potential as f32 / 1600.0;
            }
        }
        BidderType::Developer => {
            if property.land_size >= 600 {
                modifier += 0.08;
            } else {
                modifier -= 0.04;
            }
        }
        BidderType::EgoBidder => {
            modifier += 0.05;
        }
        BidderType::BargainHunter => {
            modifier -= 0.08;
        }
    }

    let market_value = market_adjusted_value(property, market);
    round_down_to_increment(
        (market_value as f32 * (0.96 + modifier)) as i64,
        BID_INCREMENT,
    )
    .max(property.guide_price)
}
