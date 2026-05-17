use serde::Deserialize;

pub type PropertyId = usize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Rough,
    Tired,
    Solid,
    Premium,
}

impl Condition {
    pub fn label(self) -> &'static str {
        match self {
            Condition::Rough => "Rough",
            Condition::Tired => "Tired",
            Condition::Solid => "Solid",
            Condition::Premium => "Premium",
        }
    }

    pub fn defect_penalty_rate(self) -> f32 {
        match self {
            Condition::Rough => 0.09,
            Condition::Tired => 0.055,
            Condition::Solid => 0.025,
            Condition::Premium => 0.01,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DealArchetype {
    RiskyFixer,
    PrettyTrap,
    LandValuePlay,
    HotSuburbFomo,
    QuietBargain,
    RenovatorBait,
    RentalHold,
    AuctionTrap,
}

impl DealArchetype {
    pub fn label(self) -> &'static str {
        match self {
            DealArchetype::RiskyFixer => "Risky Fixer",
            DealArchetype::PrettyTrap => "Pretty Trap",
            DealArchetype::LandValuePlay => "Land Value Play",
            DealArchetype::HotSuburbFomo => "Hot Suburb FOMO",
            DealArchetype::QuietBargain => "Quiet Bargain",
            DealArchetype::RenovatorBait => "Renovator Bait",
            DealArchetype::RentalHold => "Rental Hold",
            DealArchetype::AuctionTrap => "Auction Trap",
        }
    }

    pub fn lesson(self) -> &'static str {
        match self {
            DealArchetype::RiskyFixer => "Cheap entry can be erased by repairs.",
            DealArchetype::PrettyTrap => "A safe-looking home can already be fully priced.",
            DealArchetype::LandValuePlay => "The block matters more than the tired house.",
            DealArchetype::HotSuburbFomo => "Demand helps resale, but it also inflates the room.",
            DealArchetype::QuietBargain => "Soft demand can hide clean numbers.",
            DealArchetype::RenovatorBait => "Upgrade potential is not always upgrade profit.",
            DealArchetype::RentalHold => "Weak flip margin can still suit a patient holder.",
            DealArchetype::AuctionTrap => "A low guide does not mean a low reserve.",
        }
    }

    pub fn temptation(self) -> &'static str {
        match self {
            DealArchetype::RiskyFixer => "underprice the defect",
            DealArchetype::PrettyTrap => "pay for safety twice",
            DealArchetype::LandValuePlay => "judge the house instead of the land",
            DealArchetype::HotSuburbFomo => "chase the room",
            DealArchetype::QuietBargain => "ignore a dull but profitable deal",
            DealArchetype::RenovatorBait => "overcapitalise the upside",
            DealArchetype::RentalHold => "force a flip onto a hold asset",
            DealArchetype::AuctionTrap => "trust the guide price",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PropertyTemplate {
    pub id: PropertyId,
    pub address: String,
    pub suburb: String,
    pub bedrooms: u8,
    pub bathrooms: u8,
    pub condition: Condition,
    pub land_size: u32,
    pub market_value: i64,
    pub guide_price: i64,
    pub reserve_price: i64,
    pub appeal: i32,
    pub renovation_potential: i32,
    pub hidden_defect_risk: f32,
    pub holding_cost_per_week: i64,
    pub buyer_demand: i32,
    pub deal_archetype: DealArchetype,
    pub thesis: String,
    pub main_risk: String,
    pub best_strategy: String,
    pub bad_strategy: String,
    pub notes: String,
}

#[derive(Clone, Debug)]
pub struct Property {
    pub id: PropertyId,
    pub address: String,
    pub suburb: String,
    pub bedrooms: u8,
    pub bathrooms: u8,
    pub condition: Condition,
    pub land_size: u32,
    pub market_value: i64,
    pub guide_price: i64,
    pub reserve_price: i64,
    pub appeal: i32,
    pub renovation_potential: i32,
    pub hidden_defect_risk: f32,
    pub holding_cost_per_week: i64,
    pub buyer_demand: i32,
    pub deal_archetype: DealArchetype,
    pub thesis: String,
    pub main_risk: String,
    pub best_strategy: String,
    pub bad_strategy: String,
    pub notes: String,
}

impl Property {
    pub fn from_template(template: &PropertyTemplate) -> Self {
        Self {
            id: template.id,
            address: template.address.clone(),
            suburb: template.suburb.clone(),
            bedrooms: template.bedrooms,
            bathrooms: template.bathrooms,
            condition: template.condition,
            land_size: template.land_size,
            market_value: template.market_value,
            guide_price: template.guide_price,
            reserve_price: template.reserve_price,
            appeal: template.appeal,
            renovation_potential: template.renovation_potential,
            hidden_defect_risk: template.hidden_defect_risk,
            holding_cost_per_week: template.holding_cost_per_week,
            buyer_demand: template.buyer_demand,
            deal_archetype: template.deal_archetype,
            thesis: template.thesis.clone(),
            main_risk: template.main_risk.clone(),
            best_strategy: template.best_strategy.clone(),
            bad_strategy: template.bad_strategy.clone(),
            notes: template.notes.clone(),
        }
    }
}
