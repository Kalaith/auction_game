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
    let mut auction = create_auction(
        &property(),
        &market(),
        &profiles(),
        480_000,
        ResearchLevel::AgentPack,
        WalkawayStyle::Balanced,
    );
    auction.has_started = true;
    auction
}

#[test]
fn auction_clock_waits_for_the_visible_opening_call() {
    let mut auction = create_auction(
        &property(),
        &market(),
        &profiles(),
        480_000,
        ResearchLevel::AgentPack,
        WalkawayStyle::Balanced,
    );
    let time_before = auction.seconds_remaining;

    update_auction(&mut auction, 5.0);
    assert_eq!(auction.seconds_remaining, time_before);

    begin_auction_calls(&mut auction);
    update_auction(&mut auction, 1.0);
    assert!(auction.seconds_remaining < time_before);
}

#[test]
fn player_registration_receives_a_stable_visible_paddle_number() {
    let first = auction();
    let second = auction();

    assert_eq!(first.player_paddle_number(), second.player_paddle_number());
    assert!((100..900).contains(&first.player_paddle_number()));
}

#[test]
fn saved_auction_resumes_the_same_room_sequence() {
    let mut original = auction();
    for _ in 0..12 {
        update_auction(&mut original, 0.1);
    }
    let json = serde_json::to_string(&original).expect("auction should save");
    let mut restored: Auction = serde_json::from_str(&json).expect("auction should load");

    for _ in 0..80 {
        update_auction(&mut original, 0.1);
        update_auction(&mut restored, 0.1);
    }

    assert_eq!(restored.current_bid, original.current_bid);
    assert_eq!(restored.last_bidder, original.last_bidder);
    assert_eq!(restored.rng_state, original.rng_state);
    assert_eq!(restored.log.len(), original.log.len());
}

#[test]
fn a_suspended_browser_frame_cannot_consume_the_room_clock() {
    let mut auction = auction();
    let before = auction.seconds_remaining;

    update_auction(&mut auction, 30.0);

    assert!(auction.seconds_remaining >= before - 0.11);
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

#[test]
fn vendor_bid_is_declared_once_and_stays_below_reserve() {
    let mut auction = auction();
    auction.seconds_remaining = 40.0;

    assert!(should_place_vendor_bid(&auction));
    place_vendor_bid(&mut auction);

    assert!(auction.vendor_bid_used);
    assert_eq!(auction.last_bidder, Some(BidderActor::Vendor));
    assert!(auction.current_bid < auction.reserve_price);
    assert!(!should_place_vendor_bid(&auction));
}

#[test]
fn crossing_the_true_reserve_announces_on_market_without_revealing_it_early() {
    let mut auction = auction();
    auction.current_bid = auction.reserve_price - auction.bid_increment;
    assert!(!auction.on_market_announced);

    place_player_bid(&mut auction);

    assert!(auction.on_market_announced);
    assert_eq!(
        auction.bid_increment,
        crate::sim::auction_events::ON_MARKET_BID_INCREMENT
    );
    assert!(auction
        .log
        .iter()
        .any(|entry| entry.text.contains("on the market")));
}

#[test]
fn on_market_call_tightens_the_next_bid_without_changing_the_current_price() {
    let mut auction = auction();
    let opening_increment = auction.bid_increment;
    auction.current_bid = auction.reserve_price - opening_increment;

    place_player_bid(&mut auction);
    let price_at_call = auction.current_bid;

    assert_eq!(price_at_call, auction.reserve_price);
    assert_eq!(auction.next_bid(), price_at_call + 5_000);
    assert_eq!(auction.jump_bid(), price_at_call + 10_000);
}

#[test]
fn waiting_earns_a_room_read_and_the_next_bid_makes_it_stale() {
    let mut auction = auction();

    hold_player_position(&mut auction);
    assert!(auction.last_room_read.is_some());

    place_player_bid(&mut auction);
    assert!(auction.last_room_read.is_none());
}

#[test]
fn diligence_changes_room_read_precision() {
    let mut auction = auction();
    auction.player_research_level = ResearchLevel::StreetScan;
    let street_read = room_read(&auction);
    auction.player_research_level = ResearchLevel::AgentPack;
    let agent_read = room_read(&auction);

    assert!(street_read.contains(':'));
    assert!(agent_read.contains("looks"));
    assert_ne!(street_read, agent_read);
}

#[test]
fn ego_bidder_can_stretch_when_the_player_makes_it_personal() {
    let mut auction = auction();
    auction.last_bidder = Some(BidderActor::Player);
    let ego_limit = auction.bidders[1].max_price;

    assert!(bidder_effective_ceiling(&auction, 1) > ego_limit);
}

#[test]
fn first_home_buyer_stretches_only_when_the_final_call_feels_close() {
    let mut auction = auction();
    auction.bidders[0].bidder_type = BidderType::FirstHomeBuyer;
    auction.bidders[0].overbid_tendency = 0.7;
    let limit = auction.bidders[0].max_price;
    auction.temperature = AuctionTemperature::QuietRoom;
    auction.seconds_remaining = 30.0;
    assert_eq!(bidder_effective_ceiling(&auction, 0), limit);

    auction.seconds_remaining = 15.0;
    assert!(bidder_effective_ceiling(&auction, 0) > limit);
}

#[test]
fn renovator_stretches_for_rough_upside_only_in_an_emotional_room() {
    let mut auction = auction();
    auction.bidders[0].bidder_type = BidderType::Renovator;
    auction.bidders[0].overbid_tendency = 0.7;
    let limit = auction.bidders[0].max_price;
    auction.temperature = AuctionTemperature::FomoSpiral;
    auction.property.condition = Condition::Solid;
    assert_eq!(bidder_effective_ceiling(&auction, 0), limit);

    auction.property.condition = Condition::Tired;
    assert!(bidder_effective_ceiling(&auction, 0) > limit);
    auction.current_bid = limit;
    place_npc_bid(&mut auction, 0);
    assert!(auction.bidders[0].stretch_bid_used);
    assert!(auction.bidders[0].tell.contains("repair budget"));
}

#[test]
fn rational_archetypes_never_gain_an_emotional_ceiling() {
    for bidder_type in [
        BidderType::Investor,
        BidderType::Developer,
        BidderType::BargainHunter,
    ] {
        let mut auction = auction();
        auction.temperature = AuctionTemperature::FinalCall;
        auction.bidders[0].bidder_type = bidder_type;
        auction.bidders[0].overbid_tendency = 0.68;
        let limit = auction.bidders[0].max_price;

        assert_eq!(
            bidder_effective_ceiling(&auction, 0),
            limit,
            "{} should stay rational",
            bidder_type.label()
        );
    }
}

#[test]
fn quick_resolution_always_reaches_a_hammer_or_pass_in() {
    let mut auction = auction();
    stop_player_bidding(&mut auction);

    quick_resolve_auction(&mut auction);

    assert!(auction.status.is_some());
    if matches!(auction.status, Some(AuctionStatus::SoldToNpc(_))) {
        assert!(auction.current_bid >= auction.reserve_price);
    }
}

#[test]
fn passed_in_vendor_counter_can_be_accepted_below_reserve() {
    let mut auction = auction();
    auction.current_bid = auction.reserve_price - auction.bid_increment * 3;
    auction.status = Some(AuctionStatus::PassedIn);

    let offer = post_auction_offer(&auction).unwrap();
    assert_eq!(offer, auction.reserve_price - auction.bid_increment);
    assert!(accept_post_auction_offer(&mut auction));
    assert_eq!(auction.current_bid, offer);
    assert_eq!(auction.status, Some(AuctionStatus::SoldToPlayer));
}

#[test]
fn a_passed_in_buyer_can_test_a_flexible_vendor_once() {
    let mut auction = auction();
    auction.property.buyer_demand = 45;
    auction.current_bid = auction.reserve_price - 20_000;
    auction.status = Some(AuctionStatus::PassedIn);
    let tested_price = auction.current_bid;

    assert_eq!(
        test_vendor_at_passed_in_price(&mut auction),
        Some(PostAuctionTestResult::Accepted(tested_price))
    );
    assert_eq!(auction.status, Some(AuctionStatus::SoldToPlayer));
    assert!(auction.sold_post_auction);
}

#[test]
fn a_rejected_passed_in_test_leaves_the_vendor_counter_available() {
    let mut auction = auction();
    auction.property.buyer_demand = 80;
    auction.current_bid = auction.reserve_price - 30_000;
    auction.status = Some(AuctionStatus::PassedIn);

    let counter = post_auction_offer(&auction).unwrap();
    assert_eq!(
        test_vendor_at_passed_in_price(&mut auction),
        Some(PostAuctionTestResult::Rejected(counter))
    );
    assert_eq!(auction.status, Some(AuctionStatus::PassedIn));
    assert!(auction.post_auction_tested);
    assert_eq!(test_vendor_at_passed_in_price(&mut auction), None);
    assert!(accept_post_auction_offer(&mut auction));
}
