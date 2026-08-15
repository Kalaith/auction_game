use crate::model::{MaintenanceIssue, MaintenanceKind, OwnedProperty, Player};

pub fn trigger_due_maintenance(player: &mut Player) -> Vec<String> {
    let mut notices = Vec::new();
    for owned in &mut player.properties {
        if !owned.is_leased || owned.maintenance_issue.is_some() {
            continue;
        }
        if owned.weeks_held < next_maintenance_week(owned) {
            continue;
        }
        let issue = issue_for(owned);
        notices.push(format!(
            "{} needs {} ({}).",
            owned.property.address,
            issue.kind.label(),
            crate::ui::format_money(issue.repair_cost)
        ));
        owned.maintenance_issue = Some(issue);
    }
    notices
}

pub fn next_maintenance_week(owned: &OwnedProperty) -> u32 {
    6 + owned.property.id as u32 % 4 + u32::from(owned.maintenance_events_resolved) * 12
}

pub fn effective_weekly_rent(owned: &OwnedProperty) -> i64 {
    if !owned.is_leased {
        return 0;
    }
    let loss = owned
        .maintenance_issue
        .as_ref()
        .map(|issue| issue.weekly_rent_loss)
        .unwrap_or(0);
    (owned.weekly_rent - loss).max(0)
}

pub fn repair_maintenance(owned: &mut OwnedProperty) -> i64 {
    let Some(issue) = owned.maintenance_issue.take() else {
        return 0;
    };
    owned.maintenance_events_resolved += 1;
    issue.repair_cost
}

fn issue_for(owned: &OwnedProperty) -> MaintenanceIssue {
    match (owned.property.id + usize::from(owned.maintenance_events_resolved)) % 3 {
        0 => MaintenanceIssue {
            kind: MaintenanceKind::PlumbingLeak,
            repair_cost: 3_500,
            weekly_rent_loss: 80,
            description:
                "A persistent leak needs a licensed plumber before it damages the tenancy."
                    .to_string(),
        },
        1 => MaintenanceIssue {
            kind: MaintenanceKind::HeatingFailure,
            repair_cost: 5_200,
            weekly_rent_loss: 120,
            description:
                "Failed heating requires urgent replacement and a temporary rent reduction."
                    .to_string(),
        },
        _ => MaintenanceIssue {
            kind: MaintenanceKind::RoofRepair,
            repair_cost: 8_000,
            weekly_rent_loss: 160,
            description: "A roof leak is reducing rent until weatherproofing is complete."
                .to_string(),
        },
    }
}

#[cfg(test)]
mod tests;
