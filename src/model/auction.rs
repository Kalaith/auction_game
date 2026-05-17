use crate::model::{Bidder, Property, ResearchLevel, WalkawayStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BidderActor {
    Player,
    Npc(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuctionStatus {
    SoldToPlayer,
    SoldToNpc(String),
    PassedIn,
}

#[derive(Clone, Debug)]
pub struct BidLog {
    pub text: String,
    pub seconds_remaining: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug)]
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
}

impl Auction {
    pub fn is_running(&self) -> bool {
        self.status.is_none()
    }

    pub fn next_bid(&self) -> i64 {
        self.current_bid + self.bid_increment
    }
}
