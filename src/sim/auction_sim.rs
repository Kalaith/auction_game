use crate::model::{
    Auction, AuctionStatus, AuctionTemperature, BidLog, Bidder, BidderActor, BidderMood,
    BidderProfileData, BidderType, Condition, DealArchetype, MarketEvent, Property, ResearchLevel,
    WalkawayStyle,
};
use crate::sim::auction_bidders::{
    auction_temperature, bid_tell, bidder_ceiling, bidder_danger, bidder_preference,
    bidder_weakness, bidding_rhythm, exit_tell, initial_temperature, market_heat, opening_tell,
    overbid_tendency, pressure_tolerance, stretch_increments, table_clue, tell_for,
    temperature_bid_modifier, BID_INCREMENT,
};
#[cfg(test)]
use crate::sim::auction_events::{
    accept_post_auction_offer, post_auction_offer, test_vendor_at_passed_in_price, vendor_stance,
    PostAuctionTestResult, VendorStance,
};
use crate::sim::auction_events::{announce_on_market, place_vendor_bid, should_place_vendor_bid};
use crate::sim::valuation::round_down_to_increment;
use macroquad_toolkit::rng::SeededRng;

pub const AUCTION_DURATION_SECONDS: f32 = 56.0;

pub fn create_auction(
    property: &Property,
    market: &MarketEvent,
    profiles: &[BidderProfileData],
    player_walkaway_price: i64,
    player_research_level: ResearchLevel,
    walkaway_style: WalkawayStyle,
) -> Auction {
    let mut bidders = Vec::new();
    for offset in 0..3 {
        let profile = &profiles[(property.id + offset) % profiles.len()];
        let max_price = bidder_ceiling(property, market, profile);
        let pressure_tolerance = pressure_tolerance(profile.bidder_type, profile.patience);
        let overbid_tendency = overbid_tendency(profile.bidder_type, profile.aggression);
        bidders.push(Bidder {
            name: profile.name.clone(),
            bidder_type: profile.bidder_type,
            max_price,
            aggression: profile.aggression,
            patience: profile.patience,
            pressure_tolerance,
            overbid_tendency,
            reaction_timer: 1.0 + offset as f32 * 0.55,
            bid_count: 0,
            heat: 35 + (profile.aggression * 35.0) as i32,
            mood: BidderMood::Watching,
            tell: opening_tell(profile.bidder_type).to_string(),
            preference: bidder_preference(profile.bidder_type).to_string(),
            weakness: bidder_weakness(profile.bidder_type).to_string(),
            danger: bidder_danger(profile.bidder_type).to_string(),
            rhythm: bidding_rhythm(profile.bidder_type).to_string(),
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
            text: format!(
                "Registration confirmed. Opening call will be {}.",
                crate::ui::format_money(opening_bid)
            ),
            seconds_remaining: AUCTION_DURATION_SECONDS,
        }],
        player_walkaway_price,
        player_research_level,
        walkaway_style,
        temperature: initial_temperature(property, market),
        market_heat: market_heat(property, market),
        jump_bid_available: true,
        player_bid_count: 0,
        on_market_announced: false,
        vendor_bid_used: false,
        last_room_read: None,
        sold_post_auction: false,
        post_auction_tested: false,
        has_started: false,
        rng_state: auction_seed(property, market),
    }
}

pub fn begin_auction_calls(auction: &mut Auction) {
    if auction.has_started || !auction.is_running() {
        return;
    }
    auction.has_started = true;
    push_log(
        auction,
        format!(
            "Bidding is open at {}.",
            crate::ui::format_money(auction.current_bid)
        ),
    );
}

pub fn update_auction(auction: &mut Auction, dt: f32) {
    if !auction.is_running() || !auction.has_started {
        return;
    }

    let dt = dt.clamp(0.0, 0.1);
    auction.temperature = auction_temperature(auction);
    auction.seconds_remaining = (auction.seconds_remaining - dt).max(0.0);
    auction.call_timer -= dt;

    if auction.call_timer <= 0.0 {
        if should_place_vendor_bid(auction) {
            place_vendor_bid(auction);
        } else {
            let line = auctioneer_line(auction);
            push_log(auction, line);
        }
        auction.call_timer = auction_rng_range(auction, 1.6, 3.0);
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

        if auction_rng_chance(auction, chance) {
            bidder_to_place = Some(index);
            break;
        }

        auction.bidders[index].reaction_timer = auction_rng_range(auction, 1.1, 3.1);
    }

    if let Some(index) = bidder_to_place {
        place_npc_bid(auction, index);
    }

    auction.temperature = auction_temperature(auction);
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
    auction.last_room_read = None;
    auction.last_bidder = Some(BidderActor::Player);
    auction.player_bid_count += 1;
    extend_if_needed(auction);
    auction.temperature = auction_temperature(auction);

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
    announce_on_market(auction);
}

pub fn place_player_jump_bid(auction: &mut Auction) -> String {
    if !auction.is_running() || !auction.is_player_active || !auction.jump_bid_available {
        return "The assertive jump is no longer available.".to_string();
    }

    let jump_bid = auction.jump_bid();
    auction.current_bid = jump_bid;
    auction.last_room_read = None;
    auction.last_bidder = Some(BidderActor::Player);
    auction.jump_bid_available = false;
    auction.player_bid_count += 1;
    extend_if_needed(auction);

    let mut rattled = 0;
    let mut provoked = 0;
    for bidder in &mut auction.bidders {
        if !bidder.active {
            continue;
        }
        match bidder.bidder_type {
            BidderType::Investor | BidderType::BargainHunter => {
                bidder.max_price -= auction.bid_increment;
                bidder.reaction_timer += 1.4;
                bidder.heat = (bidder.heat - 18).max(10);
                bidder.mood = BidderMood::Hesitating;
                bidder.tell = "The sudden jump forces a fresh calculation.".to_string();
                rattled += 1;
            }
            BidderType::FirstHomeBuyer | BidderType::EgoBidder => {
                bidder.max_price += auction.bid_increment;
                bidder.reaction_timer = bidder.reaction_timer.min(0.7);
                bidder.heat = (bidder.heat + 18).min(100);
                bidder.mood = BidderMood::Stretching;
                bidder.tell = "The jump feels like a challenge.".to_string();
                provoked += 1;
            }
            BidderType::Renovator => {
                bidder.reaction_timer += 0.8;
                bidder.heat = (bidder.heat - 10).max(15);
                bidder.tell = "Rechecks the repair margin after your jump.".to_string();
                rattled += 1;
            }
            BidderType::Developer => {
                bidder.reaction_timer = bidder.reaction_timer.min(1.0);
                bidder.tell = "Ignores the theatre and checks the land value.".to_string();
            }
        }
    }
    auction.temperature = auction_temperature(auction);

    let effect = match (rattled, provoked) {
        (0, 0) => "The room barely reacts.",
        (_, 0) => "The jump rattles the room.",
        (0, _) => "The jump provokes the emotional bidders.",
        _ => "Some bidders recoil; emotional buyers take it personally.",
    };
    let message = format!("You assert {}. {effect}", crate::ui::format_money(jump_bid));
    push_log(auction, message.clone());
    announce_on_market(auction);
    message
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

pub fn hold_player_position(auction: &mut Auction) -> String {
    if !auction.is_running() || !auction.is_player_active {
        return "The room has already moved past your paddle.".to_string();
    }

    let read = room_read(auction);
    auction.last_room_read = Some(read.clone());
    push_log(auction, format!("You wait with the paddle down. {read}"));
    auction.call_timer = auction.call_timer.min(0.45);
    for bidder in &mut auction.bidders {
        if bidder.active {
            bidder.reaction_timer = bidder.reaction_timer.min(0.45);
        }
    }
    read
}

#[cfg(test)]
mod tests;

pub fn room_read(auction: &Auction) -> String {
    let next_bid = auction.next_bid();
    let mut best: Option<(usize, i64)> = None;

    for (index, bidder) in auction.bidders.iter().enumerate() {
        if !bidder.active {
            continue;
        }
        let ceiling = bidder_effective_ceiling(auction, index);
        let headroom = ceiling - next_bid;
        if best.is_none_or(|(_, best_headroom)| headroom < best_headroom) {
            best = Some((index, headroom));
        }
    }

    let Some((index, headroom)) = best else {
        return "No active bidder wants the next bid.".to_string();
    };
    let bidder = &auction.bidders[index];
    if auction.player_research_level == ResearchLevel::StreetScan {
        return format!("{}: {}", bidder.name, bidder.tell);
    }
    if auction.player_research_level == ResearchLevel::AgentPack {
        return if headroom <= auction.bid_increment * 2 {
            format!("{} looks uncomfortable at the next call.", bidder.name)
        } else {
            format!(
                "{} still looks composed; the ceiling is unclear.",
                bidder.name
            )
        };
    }
    if headroom < 0 {
        format!("{} is priced out if the room asks again.", bidder.name)
    } else if headroom <= auction.bid_increment {
        format!("{} is near ceiling; holding may flush them.", bidder.name)
    } else if bidder.bidder_type == BidderType::EgoBidder
        && auction.last_bidder == Some(BidderActor::Player)
    {
        format!(
            "{} reacts to your paddle; slower bidding reduces the bait.",
            bidder.name
        )
    } else if bidder.bidder_type == BidderType::Investor {
        format!(
            "{} is rational; pressure is less dangerous than price.",
            bidder.name
        )
    } else if bidder.bidder_type == BidderType::BargainHunter {
        format!(
            "{} needs value; reserve pressure can push them out.",
            bidder.name
        )
    } else {
        format!(
            "{} still has room, but their tell matters more than speed.",
            bidder.name
        )
    }
}

pub fn quick_resolve_auction(auction: &mut Auction) {
    if !auction.is_running() || auction.is_player_active {
        return;
    }

    push_log(
        auction,
        "You stay out. The remaining bidders resolve the room quickly.".to_string(),
    );

    for _ in 0..12 {
        auction.temperature = auction_temperature(auction);
        let Some(index) = quick_resolve_bidder(auction) else {
            break;
        };

        auction.seconds_remaining = auction.seconds_remaining.max(10.0);
        place_npc_bid(auction, index);
    }

    auction.seconds_remaining = 0.0;
    auction.temperature = AuctionTemperature::FinalCall;
    finish_auction(auction);
}

fn place_npc_bid(auction: &mut Auction, index: usize) {
    let previous_bid = auction.current_bid;
    let next_bid = npc_bid_amount(auction, index);
    let was_stretch = next_bid > auction.bidders[index].max_price;
    auction.current_bid = next_bid;
    auction.last_room_read = None;
    auction.last_bidder = Some(BidderActor::Npc(index));
    auction.bidders[index].bid_count += 1;
    auction.bidders[index].heat = (auction.bidders[index].heat + 12).min(100);
    if was_stretch {
        auction.bidders[index].stretch_bid_used = true;
        auction.bidders[index].mood = BidderMood::Stretching;
        auction.bidders[index].tell = tell_for(
            auction.bidders[index].bidder_type,
            BidderMood::Stretching,
            &auction.property,
        )
        .to_string();
    } else {
        auction.bidders[index].mood = BidderMood::Interested;
        auction.bidders[index].tell = bid_tell(&auction.bidders[index]).to_string();
    }
    auction.bidders[index].reaction_timer = auction_rng_range(auction, 1.2, 3.2);
    extend_if_needed(auction);
    auction.temperature = auction_temperature(auction);

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
    announce_on_market(auction);
}

fn quick_resolve_bidder(auction: &mut Auction) -> Option<usize> {
    let next_bid = auction.next_bid();
    let mut best: Option<(usize, f32)> = None;

    for index in 0..auction.bidders.len() {
        if !auction.bidders[index].active {
            continue;
        }
        if auction.last_bidder == Some(BidderActor::Npc(index)) {
            continue;
        }

        update_bidder_tell(auction, index, next_bid);
        let ceiling = bidder_effective_ceiling(auction, index);
        if next_bid > ceiling {
            retire_bidder(auction, index);
            continue;
        }

        let headroom = ((ceiling - next_bid) as f32 / 120_000.0).clamp(0.0, 0.32);
        let score = bid_chance(auction, index, next_bid)
            + headroom
            + auction.bidders[index].pressure_tolerance * 0.08
            - auction.bidders[index].bid_count as f32 * 0.025;

        if best.is_none_or(|(_, best_score)| score > best_score) {
            best = Some((index, score));
        }
    }

    let (index, score) = best?;
    if auction.current_bid < auction.reserve_price || score >= 0.42 {
        Some(index)
    } else {
        None
    }
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
        Some(BidderActor::Vendor) => {
            auction.status = Some(AuctionStatus::PassedIn);
            push_log(auction, "Auction passes in on the vendor bid.".to_string());
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

pub(super) fn push_log(auction: &mut Auction, text: String) {
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
    } else if auction.on_market_announced {
        "We are selling. The next silence can own the home.".to_string()
    } else if auction.temperature == AuctionTemperature::FomoSpiral {
        "The room is chasing the room now, not the house.".to_string()
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

    let emotional_room = matches!(
        auction.temperature,
        AuctionTemperature::FomoSpiral | AuctionTemperature::FinalCall
    );
    let can_stretch = match bidder.bidder_type {
        BidderType::FirstHomeBuyer => auction.seconds_remaining < 16.0 || emotional_room,
        BidderType::EgoBidder => auction.last_bidder == Some(BidderActor::Player),
        BidderType::Renovator => {
            emotional_room
                && matches!(
                    auction.property.condition,
                    Condition::Rough | Condition::Tired
                )
        }
        _ => emotional_room && bidder.overbid_tendency > 0.68,
    };

    if can_stretch {
        bidder.max_price + auction.bid_increment * stretch_increments(bidder.overbid_tendency)
    } else {
        bidder.max_price
    }
}

fn auction_seed(property: &Property, market: &MarketEvent) -> u64 {
    let identity = (property.id as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    identity
        ^ (property.reserve_price as u64).rotate_left(17)
        ^ (property.guide_price as u64).rotate_left(39)
        ^ u64::from(market.buyer_budget_modifier.to_bits()).rotate_left(7)
}

fn auction_rng_range(auction: &mut Auction, low: f32, high: f32) -> f32 {
    let mut rng = SeededRng::from_state(auction.rng_state);
    let value = rng.range_f32(low, high);
    auction.rng_state = rng.state();
    value
}

fn auction_rng_chance(auction: &mut Auction, probability: f32) -> bool {
    let mut rng = SeededRng::from_state(auction.rng_state);
    let result = rng.chance(probability.clamp(0.0, 1.0));
    auction.rng_state = rng.state();
    result
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
    let temperature_pull = temperature_bid_modifier(auction.temperature, bidder);
    let mut chance = (bidder.aggression * 0.42)
        + (bidder.patience * 0.10)
        + value_room
        + urgency
        + temperature_pull;

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
            if matches!(
                auction.temperature,
                AuctionTemperature::FomoSpiral | AuctionTemperature::FinalCall
            ) {
                chance -= 0.08;
            }
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

    match auction.property.deal_archetype {
        DealArchetype::HotSuburbFomo | DealArchetype::AuctionTrap => chance += 0.08,
        DealArchetype::QuietBargain if bidder.bidder_type == BidderType::BargainHunter => {
            chance += 0.12;
        }
        DealArchetype::LandValuePlay if bidder.bidder_type == BidderType::Developer => {
            chance += 0.10;
        }
        DealArchetype::PrettyTrap if bidder.bidder_type == BidderType::Investor => chance -= 0.06,
        DealArchetype::RiskyFixer if bidder.bidder_type == BidderType::FirstHomeBuyer => {
            chance -= 0.07;
        }
        _ => {}
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
        BidderType::FirstHomeBuyer
            if auction.temperature == AuctionTemperature::FomoSpiral && bidder.bid_count > 1 =>
        {
            jump_increments = 2;
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

    let previous_mood = auction.bidders[index].mood;
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
    if previous_mood != mood && matches!(mood, BidderMood::Interested | BidderMood::Hesitating) {
        let name = auction.bidders[index].name.clone();
        let clue = table_clue(auction.bidders[index].bidder_type, mood, &auction.property);
        push_log(auction, format!("{name} {clue}"));
    }
}
