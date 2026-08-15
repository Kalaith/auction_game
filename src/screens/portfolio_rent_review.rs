use crate::model::OwnedProperty;
use crate::ui::*;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RentReviewChoice {
    Renew,
    TestMarket,
}

pub(super) fn draw_rent_review_decision(
    rect: Rect,
    owned: &OwnedProperty,
    proposed_rent: i64,
) -> Option<RentReviewChoice> {
    highlight_panel(rect);
    label("Rent Review Due", rect.x + 16.0, rect.y + 30.0, 21, WARNING);
    draw_wrapped_text(
        "Keep the tenant on current terms, or test a higher rent and risk a vacancy.",
        rect.x + 16.0,
        rect.y + 58.0,
        rect.w - 32.0,
        15,
        TEXT_DIM,
    );
    if button(
        Rect::new(rect.x + 16.0, rect.y + rect.h - 46.0, 142.0, 34.0),
        &format!("RENEW {}", format_money(owned.weekly_rent)),
        true,
        ButtonTone::Secondary,
    ) {
        return Some(RentReviewChoice::Renew);
    }
    if button(
        Rect::new(rect.x + rect.w - 158.0, rect.y + rect.h - 46.0, 142.0, 34.0),
        &format!("ASK {}", format_money(proposed_rent)),
        true,
        ButtonTone::Primary,
    ) {
        return Some(RentReviewChoice::TestMarket);
    }
    None
}
