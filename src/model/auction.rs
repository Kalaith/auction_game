use crate::model::{Bidder, Property, ResearchLevel, WalkawayStyle};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum BidderActor {
    Player,
    Npc(usize),
    Vendor,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum AuctionStatus {
    SoldToPlayer,
    SoldToNpc(String),
    PassedIn,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BidLog {
    pub text: String,
    pub seconds_remaining: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum AuctionTemperature {
    QuietRoom,
    SteadyInterest,
    HeatingUp,
    FomoSpiral,
    FinalCall,
}

impl AuctionTemperature {
    pub fn label(self) -> &'static str {
        match self {
            AuctionTemperature::QuietRoom => "Quiet Room",
            AuctionTemperature::SteadyInterest => "Steady Interest",
            AuctionTemperature::HeatingUp => "Heating Up",
            AuctionTemperature::FomoSpiral => "FOMO Spiral",
            AuctionTemperature::FinalCall => "Final Call",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            AuctionTemperature::QuietRoom => {
                "The room is cautious. Price discovery is still rational."
            }
            AuctionTemperature::SteadyInterest => "Bidders are engaged, but not yet panicked.",
            AuctionTemperature::HeatingUp => {
                "Competition is pulling attention away from the numbers."
            }
            AuctionTemperature::FomoSpiral => {
                "Bidders are reacting emotionally. Good deals vanish quickly here."
            }
            AuctionTemperature::FinalCall => "Silence now decides the sale.",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Auction {
    pub property: Property,
    pub current_bid: i64,
    pub reserve_price: i64,
    pub bid_increment: i64,
    pub seconds_remaining: f32,
    pub call_timer: f32,
    pub bidders: Vec<Bidder>,
    pub last_bidder: Option<BidderActor>,
    pub is_player_active: bool,
    pub player_exit_bid: Option<i64>,
    pub overtime_count: u8,
    pub status: Option<AuctionStatus>,
    pub log: Vec<BidLog>,
    pub player_walkaway_price: i64,
    pub player_research_level: ResearchLevel,
    pub walkaway_style: WalkawayStyle,
    pub temperature: AuctionTemperature,
    pub market_heat: i32,
    #[serde(default = "jump_available_default")]
    pub jump_bid_available: bool,
    #[serde(default)]
    pub player_bid_count: u8,
    #[serde(default)]
    pub on_market_announced: bool,
    #[serde(default)]
    pub vendor_bid_used: bool,
    #[serde(default)]
    pub last_room_read: Option<String>,
    #[serde(default)]
    pub sold_post_auction: bool,
    #[serde(default)]
    pub post_auction_tested: bool,
    #[serde(default = "auction_started_default")]
    pub has_started: bool,
    #[serde(default = "auction_rng_default")]
    pub rng_state: u64,
}

impl Auction {
    pub fn is_running(&self) -> bool {
        self.status.is_none()
    }

    pub fn next_bid(&self) -> i64 {
        self.current_bid + self.bid_increment
    }

    pub fn jump_bid(&self) -> i64 {
        self.current_bid + self.bid_increment * 2
    }
}

fn jump_available_default() -> bool {
    true
}

fn auction_started_default() -> bool {
    true
}

fn auction_rng_default() -> u64 {
    0xA117_C710_5EED_2026
}
