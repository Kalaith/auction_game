use super::*;
use std::collections::HashMap;

fn property() -> Property {
    Property {
        id: 0,
        address: "10 Test Street".to_string(),
        suburb: "Westport".to_string(),
        bedrooms: 3,
        bathrooms: 1,
        condition: Condition::Tired,
        land_size: 500,
        market_value: 500_000,
        guide_price: 450_000,
        reserve_price: 470_000,
        appeal: 55,
        renovation_potential: 60,
        hidden_defect_risk: 0.1,
        holding_cost_per_week: 120,
        buyer_demand: 55,
        deal_archetype: DealArchetype::RentalHold,
        thesis: String::new(),
        main_risk: String::new(),
        best_strategy: String::new(),
        bad_strategy: String::new(),
        notes: String::new(),
    }
}

fn market() -> MarketEvent {
    MarketEvent {
        title: "Test".to_string(),
        items: Vec::new(),
        suburb_modifiers: HashMap::new(),
        renovator_modifier: 0.0,
        buyer_budget_modifier: 0.0,
        strategy_effect: String::new(),
    }
}

fn profiles() -> Vec<BidderProfileData> {
    vec![
        BidderProfileData {
            name: "Investor".to_string(),
            bidder_type: BidderType::Investor,
            aggression: 0.5,
            patience: 0.7,
            budget_bias: 0.0,
        },
        BidderProfileData {
            name: "Ego".to_string(),
            bidder_type: BidderType::EgoBidder,
            aggression: 0.8,
            patience: 0.4,
            budget_bias: 0.0,
        },
        BidderProfileData {
            name: "Hunter".to_string(),
            bidder_type: BidderType::BargainHunter,
            aggression: 0.4,
            patience: 0.8,
            budget_bias: 0.0,
        },
    ]
}

fn auction() -> Auction {
    create_auction(
        &property(),
        &market(),
        &profiles(),
        480_000,
        ResearchLevel::AgentPack,
        WalkawayStyle::Balanced,
    )
}

#[test]
fn assertive_jump_moves_two_steps_and_changes_rival_pressure() {
    let mut auction = auction();
    let opening = auction.current_bid;
    let investor_limit = auction.bidders[0].max_price;
    let ego_limit = auction.bidders[1].max_price;

    place_player_jump_bid(&mut auction);

    assert_eq!(auction.current_bid, opening + auction.bid_increment * 2);
    assert!(!auction.jump_bid_available);
    assert_eq!(
        auction.bidders[0].max_price,
        investor_limit - auction.bid_increment
    );
    assert_eq!(
        auction.bidders[1].max_price,
        ego_limit + auction.bid_increment
    );
}

#[test]
fn bid_inside_final_call_extends_the_auction() {
    let mut auction = auction();
    auction.seconds_remaining = 3.0;

    place_player_bid(&mut auction);

    assert_eq!(auction.seconds_remaining, 11.0);
    assert_eq!(auction.overtime_count, 1);
}

#[test]
fn property_passes_in_when_reserve_is_not_met() {
    let mut auction = auction();
    auction.current_bid = auction.reserve_price - auction.bid_increment;
    auction.last_bidder = Some(BidderActor::Player);

    finish_auction(&mut auction);

    assert_eq!(auction.status, Some(AuctionStatus::PassedIn));
}

#[test]
fn hammer_falls_to_the_actual_last_bidder_at_reserve() {
    let mut auction = auction();
    auction.current_bid = auction.reserve_price;
    auction.last_bidder = Some(BidderActor::Player);

    finish_auction(&mut auction);

    assert_eq!(auction.status, Some(AuctionStatus::SoldToPlayer));
}

#[test]
fn rational_bidder_does_not_gain_an_emotional_ceiling() {
    let mut auction = auction();
    auction.temperature = AuctionTemperature::FinalCall;
    let investor_limit = auction.bidders[0].max_price;

    assert_eq!(bidder_effective_ceiling(&auction, 0), investor_limit);
}
