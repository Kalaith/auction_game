use crate::app::App;
use crate::model::{ActiveRenovation, ContractorTier, OwnedProperty, UpgradeData};
use crate::screens::Screen;
use crate::sim::renovation::{quote_renovation, RenovationQuote};
use crate::sim::sale_sim::ReserveChoice;
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

pub(super) fn draw_problem_card(rect: Rect, owned: &OwnedProperty, bank_room: i64) {
    dark_panel(rect);
    label("Problem", rect.x + 16.0, rect.y + 30.0, 22, TEXT_BRIGHT);
    let (summary, color) = if let Some(project) = &owned.active_renovation {
        (
            format!(
                "{} underway. {}w left before value changes.",
                project.upgrade_name, project.weeks_remaining
            ),
            WARNING,
        )
    } else if owned.hidden_defect_discovered && !owned.has_defect_repair() {
        (
            "Structural concern found. Buyers will punish this unless repaired.".to_string(),
            NEGATIVE,
        )
    } else if owned.hidden_defect_discovered {
        (
            "Major defect handled. Presentation now matters more than fear.".to_string(),
            POSITIVE,
        )
    } else {
        (
            "No major hidden defect surfaced yet. Keep margin for surprises.".to_string(),
            WARNING,
        )
    };
    draw_wrapped_text(
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
                || format!("Bank room {}", format_money(bank_room)),
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
    draw_wrapped_text(
        &detail,
        rect.x + 16.0,
        rect.y + 112.0,
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
    draw_wrapped_text(
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
            "Start {} | +{} value",
            format_money(quote.total_cost),
            format_money(quote.value_boost)
        ),
        rect.x + 16.0,
        rect.y + 104.0,
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
        rect.y + 126.0,
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
            "Cost {}",
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

pub(super) fn draw_sell_decision(rect: Rect, position: i64, locked: bool) -> Option<ReserveChoice> {
    soft_panel(rect);
    label(
        "Sell Strategy",
        rect.x + 16.0,
        rect.y + 30.0,
        21,
        TEXT_BRIGHT,
    );
    label(
        &format!("Likely position: {}", format_money(position)),
        rect.x + 16.0,
        rect.y + 62.0,
        17,
        if position >= 0 { POSITIVE } else { NEGATIVE },
    );
    label(
        if locked {
            "Finish active work before selling."
        } else {
            "Choose reserve intent."
        },
        rect.x + 16.0,
        rect.y + 88.0,
        15,
        TEXT_DIM,
    );

    if button(
        Rect::new(rect.x + 16.0, rect.y + rect.h - 46.0, 96.0, 34.0),
        "Quick",
        !locked,
        ButtonTone::Primary,
    ) {
        return Some(ReserveChoice::Conservative);
    }
    if button(
        Rect::new(rect.x + 124.0, rect.y + rect.h - 46.0, 82.0, 34.0),
        "Fair",
        !locked,
        ButtonTone::Secondary,
    ) {
        return Some(ReserveChoice::Fair);
    }
    if button(
        Rect::new(rect.x + 218.0, rect.y + rect.h - 46.0, 100.0, 34.0),
        "Stretch",
        !locked,
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
    let target = if owned.hidden_defect_discovered && !owned.has_defect_repair() {
        "structural_repair"
    } else if !owned.has_upgrade("paint_clean") {
        "paint_clean"
    } else if !owned.has_upgrade("staging") {
        "staging"
    } else {
        "landscaping"
    };
    let upgrade = app
        .data
        .upgrades
        .iter()
        .find(|upgrade| upgrade.id == target)?;
    Some((
        upgrade,
        quote_renovation(owned, upgrade, app.selected_contractor, app.market()),
    ))
}

pub(super) fn draw_contractor_selector(app: &mut App, main: Rect, locked: bool) {
    label(
        "Contractor",
        main.x + 18.0,
        ui_height() - 60.0,
        16,
        TEXT_DIM,
    );
    let options = [
        ContractorTier::Budget,
        ContractorTier::Reliable,
        ContractorTier::Premium,
    ];
    for (index, tier) in options.iter().enumerate() {
        let selected = app.selected_contractor == *tier;
        if button(
            Rect::new(
                main.x + 116.0 + index as f32 * 82.0,
                ui_height() - 82.0,
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
