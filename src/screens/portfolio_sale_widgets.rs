use crate::app::App;
use crate::model::OwnedProperty;
use crate::sim::sale_sim::{marketing_demand_bonus, MarketingPlan, ReserveChoice};
use crate::ui::*;
use macroquad::prelude::*;

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

pub(super) fn draw_marketing_selector(app: &mut App, main: Rect, owned: &OwnedProperty) {
    label("Marketing", main.x + 744.0, main.y + 266.0, 16, TEXT_DIM);
    let options = [
        MarketingPlan::Budget,
        MarketingPlan::Standard,
        MarketingPlan::Premium,
    ];
    for (index, plan) in options.iter().enumerate() {
        let selected = app.selected_marketing_plan == *plan;
        if button(
            Rect::new(
                main.x + 840.0 + index as f32 * 90.0,
                main.y + 240.0,
                84.0,
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
