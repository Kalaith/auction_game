use crate::app::App;
use crate::model::{ActiveRenovation, ContractorTier, OwnedProperty, UpgradeData};
use crate::screens::Screen;
use crate::sim::renovation::{diagnose_property, quote_renovation, RenovationQuote};
use crate::sim::sale_sim::{marketing_demand_bonus, MarketingPlan, ReserveChoice};
use crate::ui::*;
use macroquad::prelude::*;

pub(super) fn draw_empty_portfolio(app: &mut App) {
    let rect = Rect::new(28.0, 142.0, ui_width() - 56.0, 220.0);
    soft_panel(rect);
    label(
        "No properties owned",
        rect.x + 20.0,
        rect.y + 42.0,
        26,
        TEXT_BRIGHT,
    );
    draw_wrapped_text(
        "Buy a property at auction, then decide whether repair, holding, or selling protects your cash.",
        rect.x + 20.0,
        rect.y + 84.0,
        rect.w - 40.0,
        19,
        TEXT,
    );
    if button(
        Rect::new(rect.x + 20.0, rect.y + 146.0, 164.0, 42.0),
        "Find Auctions",
        true,
        ButtonTone::Primary,
    ) {
        app.screen = Screen::PropertyList;
    }
}

pub(super) fn draw_problem_card(
    rect: Rect,
    owned: &OwnedProperty,
    bank_room: i64,
    market: &crate::model::MarketEvent,
) {
    dark_panel(rect);
    label("Diagnosis", rect.x + 16.0, rect.y + 30.0, 22, TEXT_BRIGHT);
    let diagnosis = diagnose_property(owned, market);
    let (summary, color) = if let Some(project) = &owned.active_renovation {
        (
            format!(
                "{} underway. {}w left before value changes.",
                project.upgrade_name, project.weeks_remaining
            ),
            WARNING,
        )
    } else {
        let color = if diagnosis.is_warning {
            WARNING
        } else {
            POSITIVE
        };
        (diagnosis.summary.clone(), color)
    };
    label_fit(
        &summary,
        rect.x + 16.0,
        rect.y + 62.0,
        rect.w - 32.0,
        17,
        color,
    );
    let detail = owned.active_renovation.as_ref().map_or_else(
        || {
            owned.upgrades.last().map_or_else(
                || format!("Best move: {}", diagnosis.best_move),
                |last| {
                    format!(
                        "Last: {} by {} contractor, {}w. {}",
                        last.name,
                        last.contractor.label(),
                        last.weeks_taken,
                        short_note(&last.note)
                    )
                },
            )
        },
        |project| {
            format!(
                "Paid {} | {} contractor",
                format_money(project.total_cost),
                project.contractor.label()
            )
        },
    );
    label_fit(
        &detail,
        rect.x + 16.0,
        rect.y + 96.0,
        rect.w - 32.0,
        15,
        if bank_room >= 80_000 {
            POSITIVE
        } else if bank_room >= 0 {
            WARNING
        } else {
            NEGATIVE
        },
    );
}

pub(super) fn draw_active_project_decision(rect: Rect, owned: &OwnedProperty) {
    let Some(project) = &owned.active_renovation else {
        return;
    };
    highlight_panel(rect);
    label(
        "Project Running",
        rect.x + 16.0,
        rect.y + 30.0,
        21,
        TEXT_BRIGHT,
    );
    label(
        &project.upgrade_name,
        rect.x + 16.0,
        rect.y + 58.0,
        17,
        TEXT,
    );
    let done = project.weeks_total.saturating_sub(project.weeks_remaining);
    let progress = if project.weeks_total == 0 {
        1.0
    } else {
        done as f32 / project.weeks_total as f32
    };
    let bar = Rect::new(rect.x + 16.0, rect.y + 78.0, rect.w - 32.0, 12.0);
    draw_rectangle(bar.x, bar.y, bar.w, bar.h, PANEL_DARK);
    draw_rectangle(bar.x, bar.y, bar.w * progress, bar.h, WARNING);
    draw_rectangle_lines(bar.x, bar.y, bar.w, bar.h, 1.0, PANEL_EDGE);
    label(
        &format!(
            "{} week{} remaining | {}",
            project.weeks_remaining,
            if project.weeks_remaining == 1 {
                ""
            } else {
                "s"
            },
            project_status(project)
        ),
        rect.x + 16.0,
        rect.y + 114.0,
        16,
        WARNING,
    );
    label(
        "Advance a week to move the job forward.",
        rect.x + 16.0,
        rect.y + 140.0,
        15,
        TEXT_DIM,
    );
}

pub(super) fn draw_upgrade_decision(
    rect: Rect,
    upgrade: &UpgradeData,
    quote: &RenovationQuote,
    cash: i64,
    already_done: bool,
) -> bool {
    highlight_panel(rect);
    label(
        upgrade.name.as_str(),
        rect.x + 16.0,
        rect.y + 30.0,
        21,
        TEXT_BRIGHT,
    );
    label_fit(
        quote.warning.as_str(),
        rect.x + 16.0,
        rect.y + 58.0,
        rect.w - 32.0,
        15,
        if quote.is_overcapitalized {
            WARNING
        } else {
            TEXT_DIM
        },
    );
    label(
        &format!(
            "{} | net {}",
            quote.verdict.label(),
            format_money(quote.net_effect)
        ),
        rect.x + 16.0,
        rect.y + 98.0,
        16,
        if quote.net_effect >= 0 {
            POSITIVE
        } else {
            WARNING
        },
    );
    label(
        &format!(
            "Start {} | +{} value",
            format_money(quote.total_cost),
            format_money(quote.value_boost)
        ),
        rect.x + 16.0,
        rect.y + 120.0,
        17,
        TEXT,
    );
    label(
        &format!(
            "{}w job | buffer {}",
            quote.holding_weeks,
            format_money(quote.total_cost + quote.holding_cost)
        ),
        rect.x + 16.0,
        rect.y + 142.0,
        15,
        if quote.permit_risk > 0 {
            WARNING
        } else {
            TEXT_DIM
        },
    );
    button(
        Rect::new(rect.x + rect.w - 118.0, rect.y + rect.h - 46.0, 98.0, 34.0),
        if already_done { "Done" } else { "Start" },
        !already_done && cash >= quote.total_cost,
        ButtonTone::Primary,
    )
}

pub(super) fn draw_skip_renovation(rect: Rect, owned: &OwnedProperty) {
    soft_panel(rect);
    label("Leave It Alone", rect.x + 16.0, rect.y + 30.0, 21, POSITIVE);
    draw_wrapped_text(
        "No available project earns back its cost on this home right now.",
        rect.x + 16.0,
        rect.y + 58.0,
        rect.w - 32.0,
        16,
        TEXT_DIM,
    );
    label(
        if owned.is_leased {
            "The tenant is already making this asset work."
        } else {
            "Keep the cash for leasing or another deposit."
        },
        rect.x + 16.0,
        rect.y + 112.0,
        16,
        POSITIVE,
    );
}

pub(super) fn draw_hold_decision(
    rect: Rect,
    owned: &OwnedProperty,
    campaign_finished: bool,
) -> bool {
    soft_panel(rect);
    let active = owned.active_renovation.as_ref();
    label(
        if active.is_some() {
            "Advance Work"
        } else if owned.is_leased {
            "Collect Rent & Hold"
        } else {
            "Hold 1 Week"
        },
        rect.x + 16.0,
        rect.y + 30.0,
        21,
        TEXT_BRIGHT,
    );
    let copy = active.map_or(
        "Wait for a better market pulse, but pay the carrying cost.".to_string(),
        |project| {
            format!(
                "Move {} forward. Sale stays locked until the crew finishes.",
                project.upgrade_name
            )
        },
    );
    draw_wrapped_text(
        &copy,
        rect.x + 16.0,
        rect.y + 58.0,
        rect.w - 32.0,
        16,
        TEXT_DIM,
    );
    label(
        &format!(
            "Rent {} | property cost {}",
            format_money(owned.weekly_rent),
            format_money(owned.property.holding_cost_per_week)
        ),
        rect.x + 16.0,
        rect.y + 104.0,
        18,
        WARNING,
    );
    button(
        Rect::new(rect.x + rect.w - 124.0, rect.y + rect.h - 46.0, 104.0, 34.0),
        if active.is_some() {
            "Next Week"
        } else {
            "Hold"
        },
        !campaign_finished,
        ButtonTone::Secondary,
    )
}

pub(super) fn draw_lease_decision(
    rect: Rect,
    weekly_rent: i64,
    leasing_cost: i64,
    cash: i64,
    locked: bool,
) -> bool {
    highlight_panel(rect);
    label(
        "Place A Tenant",
        rect.x + 16.0,
        rect.y + 30.0,
        21,
        TEXT_BRIGHT,
    );
    draw_wrapped_text(
        if locked {
            "Finish active work or repair the known defect before placing a tenant."
        } else {
            "Advertise the home and turn this purchase into a working portfolio asset."
        },
        rect.x + 16.0,
        rect.y + 58.0,
        rect.w - 32.0,
        16,
        TEXT_DIM,
    );
    label(
        &format!("{} / week", format_money(weekly_rent)),
        rect.x + 16.0,
        rect.y + 108.0,
        18,
        POSITIVE,
    );
    label(
        &format!("Leasing cost {}", format_money(leasing_cost)),
        rect.x + 16.0,
        rect.y + 132.0,
        15,
        TEXT_DIM,
    );
    button(
        Rect::new(rect.x + rect.w - 124.0, rect.y + rect.h - 46.0, 104.0, 34.0),
        if locked { "Repair First" } else { "Lease" },
        !locked && cash >= leasing_cost,
        ButtonTone::Primary,
    )
}

pub(super) fn draw_sell_decision(
    rect: Rect,
    position: i64,
    locked: bool,
    marketing_plan: MarketingPlan,
    cash: i64,
) -> Option<ReserveChoice> {
    soft_panel(rect);
    label(
        "Sell Strategy",
        rect.x + 16.0,
        rect.y + 30.0,
        21,
        TEXT_BRIGHT,
    );
    let campaign_cost = marketing_plan.cost();
    let projected_position = position - campaign_cost;
    let can_sell = !locked && cash >= campaign_cost;
    label(
        &format!("After campaign: {}", format_money(projected_position)),
        rect.x + 16.0,
        rect.y + 62.0,
        17,
        if projected_position >= 0 {
            POSITIVE
        } else {
            NEGATIVE
        },
    );
    label(
        &format!(
            "{} campaign | {}",
            marketing_plan.label(),
            format_money(campaign_cost)
        ),
        rect.x + 16.0,
        rect.y + 86.0,
        15,
        if cash >= campaign_cost {
            TEXT_DIM
        } else {
            WARNING
        },
    );
    label_fit(
        if locked {
            "Finish active work before selling."
        } else if cash < campaign_cost {
            "Not enough cash for the campaign."
        } else {
            marketing_plan.description()
        },
        rect.x + 16.0,
        rect.y + 110.0,
        rect.w - 32.0,
        15,
        TEXT_DIM,
    );

    if button(
        Rect::new(rect.x + 16.0, rect.y + rect.h - 46.0, 96.0, 34.0),
        "Quick",
        can_sell,
        ButtonTone::Primary,
    ) {
        return Some(ReserveChoice::Conservative);
    }
    if button(
        Rect::new(rect.x + 124.0, rect.y + rect.h - 46.0, 82.0, 34.0),
        "Fair",
        can_sell,
        ButtonTone::Secondary,
    ) {
        return Some(ReserveChoice::Fair);
    }
    if button(
        Rect::new(rect.x + 218.0, rect.y + rect.h - 46.0, 100.0, 34.0),
        "Stretch",
        can_sell,
        ButtonTone::Danger,
    ) {
        return Some(ReserveChoice::Ambitious);
    }
    None
}

pub(super) fn recommended_upgrade<'a>(
    app: &'a App,
    owned: &OwnedProperty,
) -> Option<(&'a UpgradeData, RenovationQuote)> {
    app.data
        .upgrades
        .iter()
        .filter(|upgrade| !owned.has_upgrade(&upgrade.id) && !owned.has_active_upgrade(&upgrade.id))
        .map(|upgrade| {
            let quote = quote_renovation(
                owned,
                upgrade,
                app.selected_contractor,
                app.market(),
                app.player.reputation,
            );
            let score = treatment_score(owned, upgrade, &quote);
            (upgrade, quote, score)
        })
        .filter(|(_, quote, score)| *score > 0 && quote.net_effect > 0)
        .max_by_key(|(_, _, score)| *score)
        .map(|(upgrade, quote, _)| (upgrade, quote))
}

pub(super) fn draw_contractor_selector(app: &mut App, main: Rect, locked: bool) {
    label("Contractor", main.x + 370.0, main.y + 266.0, 16, TEXT_DIM);
    let options = [
        ContractorTier::Budget,
        ContractorTier::Reliable,
        ContractorTier::Premium,
    ];
    for (index, tier) in options.iter().enumerate() {
        let selected = app.selected_contractor == *tier;
        if button(
            Rect::new(
                main.x + 468.0 + index as f32 * 82.0,
                main.y + 240.0,
                76.0,
                30.0,
            ),
            tier.label(),
            !locked,
            if selected {
                ButtonTone::Primary
            } else {
                ButtonTone::Ghost
            },
        ) {
            app.selected_contractor = *tier;
            app.status = format!("{} contractor selected.", tier.label());
        }
    }
}

pub(super) fn draw_marketing_selector(app: &mut App, main: Rect, owned: &OwnedProperty) {
    label("Marketing", main.x + 820.0, main.y + 266.0, 16, TEXT_DIM);
    let options = [
        MarketingPlan::Budget,
        MarketingPlan::Standard,
        MarketingPlan::Premium,
    ];
    for (index, plan) in options.iter().enumerate() {
        let selected = app.selected_marketing_plan == *plan;
        if button(
            Rect::new(
                main.x + 920.0 + index as f32 * 96.0,
                main.y + 240.0,
                88.0,
                30.0,
            ),
            plan.label(),
            true,
            if selected {
                ButtonTone::Primary
            } else {
                ButtonTone::Ghost
            },
        ) {
            app.selected_marketing_plan = *plan;
            let demand_bonus = marketing_demand_bonus(*plan, owned, app.market()).round() as i32;
            app.status = format!(
                "{} campaign selected: {}, demand pressure {:+}.",
                plan.label(),
                format_money(plan.cost()),
                demand_bonus
            );
        }
    }
}

fn short_note(note: &str) -> &str {
    note.char_indices()
        .nth(30)
        .map_or(note, |(index, _)| &note[..index])
}

fn project_status(project: &ActiveRenovation) -> String {
    if project.delay_weeks > 0 {
        format!("Delay +{}w", project.delay_weeks)
    } else if project.permit_risk > 0 {
        format!("Permit risk {}%", project.permit_risk)
    } else {
        project.contractor.label().to_string()
    }
}

fn treatment_score(owned: &OwnedProperty, upgrade: &UpgradeData, quote: &RenovationQuote) -> i64 {
    let mut score = quote.net_effect / 1_000;

    if owned.hidden_defect_discovered && !owned.has_defect_repair() {
        score += if upgrade.removes_defect { 120 } else { -55 };
    }
    if owned.purchase_price > owned.walkaway_price
        && upgrade.cost >= 20_000
        && !upgrade.removes_defect
    {
        score -= 45;
    }

    match owned.property.deal_archetype {
        crate::model::DealArchetype::RiskyFixer => {
            if upgrade.removes_defect {
                score += 70;
            }
            if matches!(upgrade.id.as_str(), "kitchen_refresh" | "bathroom_upgrade") {
                score -= 35;
            }
        }
        crate::model::DealArchetype::PrettyTrap => {
            if matches!(upgrade.id.as_str(), "paint_clean" | "staging") {
                score += 25;
            }
            if matches!(upgrade.id.as_str(), "kitchen_refresh" | "bathroom_upgrade") {
                score -= 70;
            }
        }
        crate::model::DealArchetype::LandValuePlay => {
            if upgrade.id == "landscaping" {
                score += 40;
            }
            if matches!(upgrade.id.as_str(), "kitchen_refresh" | "staging") {
                score -= 45;
            }
        }
        crate::model::DealArchetype::HotSuburbFomo => {
            if upgrade.id == "staging" {
                score += 45;
            }
        }
        crate::model::DealArchetype::QuietBargain => {
            if matches!(upgrade.id.as_str(), "paint_clean" | "staging") {
                score += 35;
            }
            if upgrade.cost >= 20_000 {
                score -= 40;
            }
        }
        crate::model::DealArchetype::RenovatorBait => {
            if upgrade.cost >= 20_000 {
                score -= 35;
            }
        }
        crate::model::DealArchetype::RentalHold => {
            if upgrade.removes_defect {
                score += 35;
            }
            if upgrade.id == "staging" {
                score -= 25;
            }
        }
        crate::model::DealArchetype::AuctionTrap => {
            if upgrade.cost >= 20_000 {
                score -= 55;
            }
        }
    }

    score
}
