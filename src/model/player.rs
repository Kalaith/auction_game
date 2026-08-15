use crate::model::{
    ActiveRenovation, CompletedUpgrade, MaintenanceIssue, Property, ResearchLevel, WalkawayStyle,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Player {
    pub cash: i64,
    pub debt: i64,
    pub properties: Vec<OwnedProperty>,
    pub reputation: i32,
    #[serde(default)]
    pub career: CareerRecord,
    #[serde(default)]
    pub rival_notebook: Vec<crate::model::RivalRecord>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            cash: 220_000,
            debt: 0,
            properties: Vec::new(),
            reputation: 0,
            career: CareerRecord::default(),
            rival_notebook: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CareerRecord {
    pub auctions_attended: u32,
    pub homes_bought: u32,
    pub disciplined_walkaways: u32,
    pub post_auction_buys: u32,
    pub homes_sold: u32,
    pub realized_profit: i64,
    #[serde(default)]
    pub unused_registrations: u32,
}

impl CareerRecord {
    pub fn record_unused_registrations(&mut self, unused: u8) {
        self.unused_registrations += u32::from(unused.min(2));
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OwnedProperty {
    pub property: Property,
    pub purchase_price: i64,
    pub purchase_fees: i64,
    pub deposit_paid: i64,
    pub debt: i64,
    pub walkaway_price: i64,
    pub research_level: ResearchLevel,
    pub walkaway_style: WalkawayStyle,
    pub weeks_held: u32,
    pub active_renovation: Option<ActiveRenovation>,
    pub upgrades: Vec<CompletedUpgrade>,
    pub hidden_defect_discovered: bool,
    #[serde(default)]
    pub is_leased: bool,
    #[serde(default)]
    pub weekly_rent: i64,
    #[serde(default)]
    pub leasing_weeks_remaining: u8,
    #[serde(default)]
    pub next_rent_review_week: u32,
    #[serde(default)]
    pub rent_received: i64,
    #[serde(default)]
    pub operating_spend: i64,
    #[serde(default)]
    pub maintenance_issue: Option<MaintenanceIssue>,
    #[serde(default)]
    pub maintenance_events_resolved: u8,
}

impl OwnedProperty {
    pub fn new(
        property: Property,
        purchase_price: i64,
        purchase_fees: i64,
        deposit_paid: i64,
        debt: i64,
        walkaway_price: i64,
        research_level: ResearchLevel,
        walkaway_style: WalkawayStyle,
    ) -> Self {
        let hidden_defect_discovered = property.hidden_defect_risk >= 0.28
            || (property.hidden_defect_risk >= 0.18 && property.id % 2 == 1);
        Self {
            property,
            purchase_price,
            purchase_fees,
            deposit_paid,
            debt,
            walkaway_price,
            research_level,
            walkaway_style,
            weeks_held: 0,
            active_renovation: None,
            upgrades: Vec::new(),
            hidden_defect_discovered,
            is_leased: false,
            weekly_rent: 0,
            leasing_weeks_remaining: 0,
            next_rent_review_week: 0,
            rent_received: 0,
            operating_spend: 0,
            maintenance_issue: None,
            maintenance_events_resolved: 0,
        }
    }

    pub fn has_upgrade(&self, upgrade_id: &str) -> bool {
        self.upgrades
            .iter()
            .any(|upgrade| upgrade.upgrade_id == upgrade_id)
    }

    pub fn has_active_upgrade(&self, upgrade_id: &str) -> bool {
        self.active_renovation
            .as_ref()
            .is_some_and(|project| project.upgrade_id == upgrade_id)
    }

    pub fn has_defect_repair(&self) -> bool {
        self.upgrades.iter().any(|upgrade| upgrade.removes_defect)
    }

    pub fn upgrade_spend(&self) -> i64 {
        self.upgrades
            .iter()
            .map(|upgrade| upgrade.actual_cost)
            .sum()
    }

    pub fn holding_spend(&self) -> i64 {
        i64::from(self.weeks_held) * self.property.holding_cost_per_week
    }

    pub fn rental_profit(&self) -> i64 {
        self.rent_received - self.operating_spend
    }
}

#[cfg(test)]
mod tests;
