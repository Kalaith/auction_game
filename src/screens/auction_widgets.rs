use crate::model::BidderMood;
use crate::ui::*;
use macroquad::prelude::*;

pub fn bid_guidance(
    margin: i64,
    cash_after: i64,
    next_bid: i64,
    walkaway_price: i64,
) -> &'static str {
    if next_bid > walkaway_price {
        "This is above your walk-away reminder. Only bid if you are choosing risk on purpose."
    } else if margin < 0 {
        "The next bid is likely underwater before renovation surprises. Stopping is rational."
    } else if cash_after < 18_000 {
        "You can settle, but the cash buffer is thin. Renovation choices will be constrained."
    } else if margin > 35_000 {
        "The next bid still leaves a useful buffer after fees. This is a defensible push."
    } else {
        "The deal is still playable, but each bid is now buying less margin."
    }
}

pub fn guidance_color(margin: i64, cash_after: i64, next_bid: i64, walkaway_price: i64) -> Color {
    if next_bid > walkaway_price || margin < 0 || cash_after < 18_000 {
        WARNING
    } else {
        TEXT_DIM
    }
}

pub fn money_color(value: i64) -> Color {
    if value < 0 {
        NEGATIVE
    } else if value < 20_000 {
        WARNING
    } else {
        POSITIVE
    }
}

pub fn mood_color(mood: BidderMood) -> Color {
    match mood {
        BidderMood::Watching => TEXT_DIM,
        BidderMood::Interested => POSITIVE,
        BidderMood::Hesitating => WARNING,
        BidderMood::Stretching => NEGATIVE,
        BidderMood::Out => NEGATIVE,
    }
}

pub fn draw_heat_bar(rect: Rect, heat: i32) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_DARK);
    let heat = heat.clamp(0, 100);
    let color = if heat > 80 {
        NEGATIVE
    } else if heat > 55 {
        WARNING
    } else {
        POSITIVE
    };
    draw_rectangle(rect.x, rect.y, rect.w * heat as f32 / 100.0, rect.h, color);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, PANEL_EDGE);
}
