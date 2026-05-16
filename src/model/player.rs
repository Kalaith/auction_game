use crate::model::{ActiveRenovation, CompletedUpgrade, Property};

#[derive(Clone, Debug)]
pub struct Player {
    pub cash: i64,
    pub debt: i64,
    pub properties: Vec<OwnedProperty>,
    pub reputation: i32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            cash: 150_000,
            debt: 0,
            properties: Vec::new(),
            reputation: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OwnedProperty {
    pub property: Property,
    pub purchase_price: i64,
    pub purchase_fees: i64,
    pub deposit_paid: i64,
    pub debt: i64,
    pub walkaway_price: i64,
    pub weeks_held: u32,
    pub active_renovation: Option<ActiveRenovation>,
    pub upgrades: Vec<CompletedUpgrade>,
    pub hidden_defect_discovered: bool,
}

impl OwnedProperty {
    pub fn new(
        property: Property,
        purchase_price: i64,
        purchase_fees: i64,
        deposit_paid: i64,
        debt: i64,
        walkaway_price: i64,
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
            weeks_held: 0,
            active_renovation: None,
            upgrades: Vec::new(),
            hidden_defect_discovered,
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
}
