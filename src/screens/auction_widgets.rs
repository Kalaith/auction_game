use crate::model::{AuctionTemperature, BidderMood};
use crate::ui::*;
use macroquad::prelude::*;

pub fn bid_verdict(
    margin: i64,
    cash_after: i64,
    cash_buffer_target: i64,
    next_bid: i64,
    walkaway_price: i64,
) -> &'static str {
    if next_bid > walkaway_price {
        "Break Plan"
    } else if margin < 0 {
        "Bad Deal"
    } else if cash_after < cash_buffer_target {
        "Thin Cash"
    } else if margin > 35_000 {
        "Safe Bid"
    } else {
        "Thin Margin"
    }
}

pub fn guidance_color(
    margin: i64,
    cash_after: i64,
    cash_buffer_target: i64,
    next_bid: i64,
    walkaway_price: i64,
) -> Color {
    if next_bid > walkaway_price || margin < 0 || cash_after < cash_buffer_target {
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

pub fn temperature_color(temperature: AuctionTemperature) -> Color {
    match temperature {
        AuctionTemperature::QuietRoom => POSITIVE,
        AuctionTemperature::SteadyInterest => crate::ui::BLUE,
        AuctionTemperature::HeatingUp => WARNING,
        AuctionTemperature::FomoSpiral | AuctionTemperature::FinalCall => NEGATIVE,
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
