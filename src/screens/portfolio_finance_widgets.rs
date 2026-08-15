use crate::model::OwnedProperty;
use crate::ui::*;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum LoanAction {
    PayDown,
    Refinance,
}

pub(super) fn draw_loan_control(
    rect: Rect,
    owned: &OwnedProperty,
    cash: i64,
    refinance_capacity: i64,
    lvr_percent: f32,
) -> Option<LoanAction> {
    dark_panel(rect);
    label_fit(
        &format!("LOAN · {lvr_percent:.0}% LVR"),
        rect.x + 14.0,
        rect.y + 22.0,
        rect.w - 174.0,
        13,
        TEXT_DIM,
    );
    label(
        &format_money(owned.debt),
        rect.x + 14.0,
        rect.y + 50.0,
        24,
        TEXT_BRIGHT,
    );
    let refinance_note = if refinance_capacity >= 10_000 {
        format!("Refi {}", format_money(refinance_capacity))
    } else if owned.weeks_held < 4 {
        "Refi after 4w".to_string()
    } else {
        "No refi headroom".to_string()
    };
    label_fit(
        &refinance_note,
        rect.x + 14.0,
        rect.y + 72.0,
        rect.w - 174.0,
        13,
        POSITIVE,
    );
    if button(
        Rect::new(rect.x + rect.w - 154.0, rect.y + 10.0, 138.0, 30.0),
        "Pay Down $10k",
        owned.debt > 0 && cash >= 10_000,
        ButtonTone::Secondary,
    ) {
        return Some(LoanAction::PayDown);
    }
    if button(
        Rect::new(rect.x + rect.w - 154.0, rect.y + 44.0, 138.0, 30.0),
        "Release Equity",
        refinance_capacity >= 10_000,
        ButtonTone::Primary,
    ) {
        return Some(LoanAction::Refinance);
    }
    None
}
