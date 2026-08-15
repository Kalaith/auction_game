use super::App;
use crate::model::PropertyId;
use crate::sim::maintenance::repair_maintenance;
use crate::sim::rental::{end_tenancy, leasing_cost, weekly_rent_for_owned};
use crate::ui::format_money;

impl App {
    pub(crate) fn lease_property(&mut self, property_id: PropertyId) {
        let Some(index) = self
            .player
            .properties
            .iter()
            .position(|owned| owned.property.id == property_id)
        else {
            return;
        };
        if self.player.properties[index].is_leased {
            return;
        }
        if self.player.properties[index].active_renovation.is_some() {
            self.status = "Finish the renovation before placing a tenant.".to_string();
            return;
        }
        if self.player.properties[index].hidden_defect_discovered
            && !self.player.properties[index].has_defect_repair()
        {
            self.status = "Repair the known structural risk before placing a tenant.".to_string();
            return;
        }
        let rent = weekly_rent_for_owned(&self.player.properties[index], self.market());
        let fee = leasing_cost(rent);
        if self.player.cash < fee {
            self.status = format!("Need {} for advertising and leasing.", format_money(fee));
            return;
        }
        self.player.cash -= fee;
        self.player.properties[index].is_leased = true;
        self.player.properties[index].weekly_rent = rent;
        self.status = format!(
            "Tenant placed at {} per week. Leasing cost {}.",
            format_money(rent),
            format_money(fee)
        );
    }

    pub(crate) fn end_property_tenancy(&mut self, property_id: PropertyId) {
        let Some(index) = self
            .player
            .properties
            .iter()
            .position(|owned| owned.property.id == property_id)
        else {
            return;
        };
        let turnover_cost = self.player.properties[index].weekly_rent;
        if !self.player.properties[index].is_leased || self.player.cash < turnover_cost {
            return;
        }
        let turnover_cost = end_tenancy(&mut self.player.properties[index]);
        self.player.cash -= turnover_cost;
        self.status = format!(
            "Tenancy ended for {}. Turnover cost {} paid; renovation is now available.",
            self.player.properties[index].property.address,
            format_money(turnover_cost)
        );
    }

    pub(crate) fn repair_property_maintenance(&mut self, property_id: PropertyId) {
        let Some(index) = self
            .player
            .properties
            .iter()
            .position(|owned| owned.property.id == property_id)
        else {
            return;
        };
        let Some(issue) = self.player.properties[index].maintenance_issue.as_ref() else {
            return;
        };
        if self.player.cash < issue.repair_cost {
            self.status = format!(
                "Need {} to repair the issue.",
                format_money(issue.repair_cost)
            );
            return;
        }
        let issue_name = issue.kind.label();
        let cost = repair_maintenance(&mut self.player.properties[index]);
        self.player.cash -= cost;
        self.status = format!(
            "{} repaired for {}. Full rent is restored.",
            issue_name,
            format_money(cost)
        );
    }
}
