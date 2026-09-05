use crate::model::{BidderProfileData, MarketEvent, PropertyTemplate, UpgradeData};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct GameData {
    pub properties: Vec<PropertyTemplate>,
    pub bidder_profiles: Vec<BidderProfileData>,
    pub upgrades: Vec<UpgradeData>,
    pub market_events: Vec<MarketEvent>,
}

impl GameData {
    pub fn load() -> Self {
        macroquad_toolkit::include_json!("../../assets/game_data.json")
            .expect("assets/game_data.json should be valid game data")
    }
}

#[cfg(test)]
mod tests;
