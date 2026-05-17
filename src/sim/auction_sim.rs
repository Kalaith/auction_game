use crate::model::{
    Auction, AuctionStatus, AuctionTemperature, BidLog, Bidder, BidderActor, BidderMood,
    BidderProfileData, BidderType, Condition, DealArchetype, MarketEvent, Property, ResearchLevel,
    WalkawayStyle,
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
            text: format!("Auction opens at {}.", crate::ui::format_money(opening_bid)),
            seconds_remaining: AUCTION_DURATION_SECONDS,
        }],
        player_walkaway_price,
        player_research_level,
        walkaway_style,
        temperature: initial_temperature(property, market),
        market_heat: market_heat(property, market),
    }
}

pub fn update_auction(auction: &mut Auction, dt: f32) {
    if !auction.is_running() {
        return;
    }

    auction.temperature = auction_temperature(auction);
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
    auction.last_bidder = Some(BidderActor::Player);
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
    push_log(auction, format!("You hold. {read}"));
    auction.call_timer = auction.call_timer.min(0.45);
    for bidder in &mut auction.bidders {
        if bidder.active {
            bidder.reaction_timer = bidder.reaction_timer.min(0.45);
        }
    }
    read
}

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
        BidderType::FirstHomeBuyer => {
            if auction.temperature == AuctionTemperature::FomoSpiral && bidder.bid_count > 1 {
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

fn table_clue(bidder_type: BidderType, mood: BidderMood, property: &Property) -> &'static str {
    match (bidder_type, mood) {
        (BidderType::Investor, BidderMood::Hesitating) => "pauses when the margin gets thin.",
        (BidderType::Renovator, BidderMood::Interested)
            if matches!(property.condition, Condition::Rough | Condition::Tired) =>
        {
            "studies the repair notes again."
        }
        (BidderType::Developer, BidderMood::Interested) if property.land_size >= 600 => {
            "keeps checking the land size."
        }
        (BidderType::BargainHunter, BidderMood::Hesitating) => "waits for the room to blink.",
        (BidderType::FirstHomeBuyer, BidderMood::Interested) => {
            "moves quickly on the family appeal."
        }
        (BidderType::EgoBidder, BidderMood::Interested) => {
            "watches your paddle more than the house."
        }
        (_, BidderMood::Interested) => "leans back into the auction.",
        (_, BidderMood::Hesitating) => "hesitates.",
        _ => "keeps watching.",
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

    match property.deal_archetype {
        DealArchetype::RiskyFixer if profile.bidder_type == BidderType::Renovator => {
            modifier += 0.04;
        }
        DealArchetype::PrettyTrap if profile.bidder_type == BidderType::FirstHomeBuyer => {
            modifier += 0.03;
        }
        DealArchetype::LandValuePlay if profile.bidder_type == BidderType::Developer => {
            modifier += 0.06;
        }
        DealArchetype::HotSuburbFomo if profile.bidder_type == BidderType::EgoBidder => {
            modifier += 0.05;
        }
        DealArchetype::QuietBargain if profile.bidder_type == BidderType::BargainHunter => {
            modifier += 0.05;
        }
        DealArchetype::RenovatorBait if profile.bidder_type == BidderType::Renovator => {
            modifier += 0.05;
        }
        DealArchetype::RentalHold if profile.bidder_type == BidderType::Investor => {
            modifier += 0.04;
        }
        DealArchetype::AuctionTrap => {
            modifier += 0.03;
        }
        _ => {}
    }

    let market_value = market_adjusted_value(property, market);
    round_down_to_increment(
        (market_value as f32 * (0.96 + modifier)) as i64,
        BID_INCREMENT,
    )
    .max(property.guide_price)
}

fn market_heat(property: &Property, market: &MarketEvent) -> i32 {
    (property.buyer_demand + (market.suburb_modifier(&property.suburb) * 100.0) as i32)
        .clamp(20, 98)
}

fn initial_temperature(property: &Property, market: &MarketEvent) -> AuctionTemperature {
    let heat = market_heat(property, market);
    if heat >= 82 || property.deal_archetype == DealArchetype::AuctionTrap {
        AuctionTemperature::HeatingUp
    } else if heat >= 58 {
        AuctionTemperature::SteadyInterest
    } else {
        AuctionTemperature::QuietRoom
    }
}

fn auction_temperature(auction: &Auction) -> AuctionTemperature {
    if auction.seconds_remaining < 9.0 {
        return AuctionTemperature::FinalCall;
    }

    let active_bidders = auction
        .bidders
        .iter()
        .filter(|bidder| bidder.active)
        .count() as i32;
    let reserve_met = auction.current_bid >= auction.reserve_price;
    let pressure = 100.0 - auction.seconds_remaining / AUCTION_DURATION_SECONDS * 100.0;
    let mut heat = auction.market_heat + active_bidders * 7 + (pressure * 0.28) as i32;

    if reserve_met {
        heat += 10;
    }
    if auction.overtime_count > 0 {
        heat += 14;
    }
    if matches!(
        auction.property.deal_archetype,
        DealArchetype::HotSuburbFomo | DealArchetype::AuctionTrap
    ) {
        heat += 8;
    }

    if heat >= 112 {
        AuctionTemperature::FomoSpiral
    } else if heat >= 86 {
        AuctionTemperature::HeatingUp
    } else if heat >= 58 {
        AuctionTemperature::SteadyInterest
    } else {
        AuctionTemperature::QuietRoom
    }
}

fn temperature_bid_modifier(temperature: AuctionTemperature, bidder: &Bidder) -> f32 {
    match temperature {
        AuctionTemperature::QuietRoom => -0.04 + bidder.patience * 0.04,
        AuctionTemperature::SteadyInterest => 0.02,
        AuctionTemperature::HeatingUp => 0.07 * bidder.pressure_tolerance,
        AuctionTemperature::FomoSpiral => 0.12 * bidder.pressure_tolerance,
        AuctionTemperature::FinalCall => 0.16 * bidder.pressure_tolerance,
    }
}

fn pressure_tolerance(bidder_type: BidderType, patience: f32) -> f32 {
    let base = match bidder_type {
        BidderType::FirstHomeBuyer => 0.70,
        BidderType::Investor => 0.42,
        BidderType::Renovator => 0.62,
        BidderType::Developer => 0.78,
        BidderType::EgoBidder => 0.92,
        BidderType::BargainHunter => 0.34,
    };
    ((base + patience) * 0.5).clamp(0.20, 0.96)
}

fn overbid_tendency(bidder_type: BidderType, aggression: f32) -> f32 {
    let base = match bidder_type {
        BidderType::FirstHomeBuyer => 0.58,
        BidderType::Investor => 0.22,
        BidderType::Renovator => 0.46,
        BidderType::Developer => 0.36,
        BidderType::EgoBidder => 0.86,
        BidderType::BargainHunter => 0.18,
    };
    ((base + aggression) * 0.5).clamp(0.05, 0.96)
}

fn stretch_increments(overbid_tendency: f32) -> i64 {
    if overbid_tendency >= 0.72 {
        2
    } else {
        1
    }
}

fn bidder_preference(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "finished family appeal",
        BidderType::Investor => "yield and margin",
        BidderType::Renovator => "rough houses with upside",
        BidderType::Developer => "large blocks",
        BidderType::EgoBidder => "visible wins",
        BidderType::BargainHunter => "quiet rooms",
    }
}

fn bidder_weakness(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "stretches late",
        BidderType::Investor => "will not chase emotion",
        BidderType::Renovator => "underweights ugly defects",
        BidderType::Developer => "ignores small blocks",
        BidderType::EgoBidder => "takes counter-bids personally",
        BidderType::BargainHunter => "folds once value buffer vanishes",
    }
}

fn bidder_danger(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "strong on pretty homes",
        BidderType::Investor => "accurate valuation",
        BidderType::Renovator => "sees upside others fear",
        BidderType::Developer => "can jump the price",
        BidderType::EgoBidder => "creates FOMO",
        BidderType::BargainHunter => "patient and hard to read",
    }
}

fn bidding_rhythm(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "fast then anxious",
        BidderType::Investor => "slow and measured",
        BidderType::Renovator => "wakes up on defects",
        BidderType::Developer => "jumps early",
        BidderType::EgoBidder => "answers quickly",
        BidderType::BargainHunter => "waits for weakness",
    }
}
