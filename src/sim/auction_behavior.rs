use crate::model::{
    Auction, AuctionTemperature, BidderActor, BidderType, Condition, DealArchetype, MarketEvent,
    Property,
};
use crate::sim::auction_bidders::{stretch_increments, temperature_bid_modifier};
use macroquad_toolkit::rng::SeededRng;

pub(super) fn bidder_effective_ceiling(auction: &Auction, index: usize) -> i64 {
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

pub(super) fn auction_seed(property: &Property, market: &MarketEvent) -> u64 {
    let identity = (property.id as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    identity
        ^ (property.reserve_price as u64).rotate_left(17)
        ^ (property.guide_price as u64).rotate_left(39)
        ^ u64::from(market.buyer_budget_modifier.to_bits()).rotate_left(7)
}

pub(super) fn auction_rng_range(auction: &mut Auction, low: f32, high: f32) -> f32 {
    let mut rng = SeededRng::from_state(auction.rng_state);
    let value = rng.range_f32(low, high);
    auction.rng_state = rng.state();
    value
}

pub(super) fn auction_rng_chance(auction: &mut Auction, probability: f32) -> bool {
    let mut rng = SeededRng::from_state(auction.rng_state);
    let result = rng.chance(probability.clamp(0.0, 1.0));
    auction.rng_state = rng.state();
    result
}

pub(super) fn bid_chance(auction: &Auction, index: usize, next_bid: i64) -> f32 {
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

pub(super) fn npc_bid_amount(auction: &Auction, index: usize) -> i64 {
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
