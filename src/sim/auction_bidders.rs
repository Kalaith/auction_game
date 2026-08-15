use crate::model::{
    Auction, AuctionTemperature, Bidder, BidderMood, BidderProfileData, BidderType, Condition,
    DealArchetype, MarketEvent, Property,
};
use crate::sim::valuation::{market_adjusted_value, round_down_to_increment};

pub(super) const BID_INCREMENT: i64 = 10_000;

pub(super) fn opening_tell(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "Clutching a pre-approval letter.",
        BidderType::Investor => "Running numbers before every call.",
        BidderType::Renovator => "Inspecting cracks and smiling anyway.",
        BidderType::Developer => "Only watching the land component.",
        BidderType::EgoBidder => "Standing too close to the auctioneer.",
        BidderType::BargainHunter => "Waiting for someone else to blink.",
    }
}

pub(super) fn exit_tell(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "Partner says no more.",
        BidderType::Investor => "Yield no longer works.",
        BidderType::Renovator => "Repair budget is gone.",
        BidderType::Developer => "Land value ceiling reached.",
        BidderType::EgoBidder => "Pride finally gets expensive.",
        BidderType::BargainHunter => "No longer a bargain.",
    }
}

pub(super) fn bid_tell(bidder: &Bidder) -> &'static str {
    match bidder.bidder_type {
        BidderType::FirstHomeBuyer => "Bids fast, then looks worried.",
        BidderType::Investor => "Raises only after checking the margin.",
        BidderType::Renovator => "Keeps bidding through cosmetic damage.",
        BidderType::Developer => "Uses big jumps to clear weak bidders.",
        BidderType::EgoBidder => "Answers your bid almost personally.",
        BidderType::BargainHunter => "Bids reluctantly and hates it.",
    }
}

pub(super) fn tell_for(
    bidder_type: BidderType,
    mood: BidderMood,
    property: &Property,
) -> &'static str {
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

pub(super) fn table_clue(
    bidder_type: BidderType,
    mood: BidderMood,
    property: &Property,
) -> &'static str {
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

pub(super) fn bidder_ceiling(
    property: &Property,
    market: &MarketEvent,
    profile: &BidderProfileData,
) -> i64 {
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
        (market_value as f32 * (0.92 + modifier)) as i64,
        BID_INCREMENT,
    )
    .max(property.guide_price)
}

pub(super) fn market_heat(property: &Property, market: &MarketEvent) -> i32 {
    (property.buyer_demand + (market.suburb_modifier(&property.suburb) * 100.0) as i32)
        .clamp(20, 98)
}

pub(super) fn initial_temperature(property: &Property, market: &MarketEvent) -> AuctionTemperature {
    let heat = market_heat(property, market);
    if heat >= 82 || property.deal_archetype == DealArchetype::AuctionTrap {
        AuctionTemperature::HeatingUp
    } else if heat >= 58 {
        AuctionTemperature::SteadyInterest
    } else {
        AuctionTemperature::QuietRoom
    }
}

pub(super) fn auction_temperature(auction: &Auction) -> AuctionTemperature {
    if auction.seconds_remaining < 9.0 {
        return AuctionTemperature::FinalCall;
    }

    let active_bidders = auction
        .bidders
        .iter()
        .filter(|bidder| bidder.active)
        .count() as i32;
    let reserve_met = auction.current_bid >= auction.reserve_price;
    let pressure =
        100.0 - auction.seconds_remaining / super::auction_sim::AUCTION_DURATION_SECONDS * 100.0;
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

pub(super) fn temperature_bid_modifier(temperature: AuctionTemperature, bidder: &Bidder) -> f32 {
    match temperature {
        AuctionTemperature::QuietRoom => -0.04 + bidder.patience * 0.04,
        AuctionTemperature::SteadyInterest => 0.02,
        AuctionTemperature::HeatingUp => 0.07 * bidder.pressure_tolerance,
        AuctionTemperature::FomoSpiral => 0.12 * bidder.pressure_tolerance,
        AuctionTemperature::FinalCall => 0.16 * bidder.pressure_tolerance,
    }
}

pub(super) fn pressure_tolerance(bidder_type: BidderType, patience: f32) -> f32 {
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

pub(super) fn overbid_tendency(bidder_type: BidderType, aggression: f32) -> f32 {
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

pub(super) fn stretch_increments(overbid_tendency: f32) -> i64 {
    if overbid_tendency >= 0.72 {
        2
    } else {
        1
    }
}

pub(super) fn bidder_preference(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "finished family appeal",
        BidderType::Investor => "yield and margin",
        BidderType::Renovator => "rough houses with upside",
        BidderType::Developer => "large blocks",
        BidderType::EgoBidder => "visible wins",
        BidderType::BargainHunter => "quiet rooms",
    }
}

pub(super) fn bidder_weakness(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "stretches late",
        BidderType::Investor => "will not chase emotion",
        BidderType::Renovator => "underweights ugly defects",
        BidderType::Developer => "ignores small blocks",
        BidderType::EgoBidder => "takes counter-bids personally",
        BidderType::BargainHunter => "folds once value buffer vanishes",
    }
}

pub(super) fn bidder_danger(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "strong on pretty homes",
        BidderType::Investor => "accurate valuation",
        BidderType::Renovator => "sees upside others fear",
        BidderType::Developer => "can jump the price",
        BidderType::EgoBidder => "creates FOMO",
        BidderType::BargainHunter => "patient and hard to read",
    }
}

pub(super) fn bidding_rhythm(bidder_type: BidderType) -> &'static str {
    match bidder_type {
        BidderType::FirstHomeBuyer => "fast then anxious",
        BidderType::Investor => "slow and measured",
        BidderType::Renovator => "wakes up on defects",
        BidderType::Developer => "jumps early",
        BidderType::EgoBidder => "answers quickly",
        BidderType::BargainHunter => "waits for weakness",
    }
}
