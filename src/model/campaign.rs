use serde::{Deserialize, Serialize};

pub const CAMPAIGN_GOAL_NET_WORTH: i64 = 240_000;
pub const CAMPAIGN_GOAL_PROPERTIES: usize = 3;
pub const CAMPAIGN_GOAL_WEEKLY_RENT: i64 = 1_500;
pub const CAMPAIGN_MAX_WEEKS: u32 = 24;
pub const WEEKLY_AUCTION_REGISTRATIONS: u8 = 2;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CampaignStatus {
    Active,
    Won,
    Failed,
}

impl CampaignStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "Campaign active",
            Self::Won => "Campaign won",
            Self::Failed => "Campaign failed",
        }
    }

    pub fn is_finished(self) -> bool {
        self != Self::Active
    }
}
