use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BidderType {
    FirstHomeBuyer,
    Investor,
    Renovator,
    Developer,
    EgoBidder,
    BargainHunter,
}

impl BidderType {
    pub fn label(self) -> &'static str {
        match self {
            BidderType::FirstHomeBuyer => "First Home Buyer",
            BidderType::Investor => "Investor",
            BidderType::Renovator => "Renovator",
            BidderType::Developer => "Developer",
            BidderType::EgoBidder => "Ego Bidder",
            BidderType::BargainHunter => "Bargain Hunter",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum BidderMood {
    Watching,
    Interested,
    Hesitating,
    Stretching,
    Out,
}

impl BidderMood {
    pub fn label(self) -> &'static str {
        match self {
            BidderMood::Watching => "watching",
            BidderMood::Interested => "interested",
            BidderMood::Hesitating => "hesitating",
            BidderMood::Stretching => "stretching",
            BidderMood::Out => "out",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BidderProfileData {
    pub name: String,
    pub bidder_type: BidderType,
    pub aggression: f32,
    pub patience: f32,
    pub budget_bias: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bidder {
    pub name: String,
    pub bidder_type: BidderType,
    pub max_price: i64,
    pub aggression: f32,
    pub patience: f32,
    pub pressure_tolerance: f32,
    pub overbid_tendency: f32,
    pub reaction_timer: f32,
    pub bid_count: u8,
    pub heat: i32,
    pub mood: BidderMood,
    pub tell: String,
    pub preference: String,
    pub weakness: String,
    pub danger: String,
    pub rhythm: String,
    pub active: bool,
    pub has_logged_exit: bool,
    pub stretch_bid_used: bool,
}
