use crate::model::Auction;
use crate::screens::auction_widgets::temperature_color;
use crate::sim::auction_sim::AUCTION_DURATION_SECONDS;
use crate::ui::*;
use macroquad::prelude::*;

pub(super) fn draw_auction_property_panel(
    rect: Rect,
    auction: &Auction,
    reserve_estimate: i64,
    cash: i64,
    bank_room: i64,
    margin: i64,
    rental_cashflow: i64,
) {
    soft_panel(rect);
    draw_house_art(
        Rect::new(rect.x + 14.0, rect.y + 14.0, rect.w - 28.0, 154.0),
        &auction.property,
    );
    label(
        &auction.property.address,
        rect.x + 16.0,
        rect.y + 204.0,
        22,
        TEXT_BRIGHT,
    );
    label(
        &auction.property.suburb,
        rect.x + 16.0,
        rect.y + 230.0,
        17,
        TEXT_DIM,
    );
    let rows = [
        ("Reserve estimate", reserve_estimate),
        ("Walk-away", auction.player_walkaway_price),
        ("Cash to settle", cash),
        ("Bank room", bank_room),
        ("Margin after fees", margin),
        ("Rental cashflow / wk", rental_cashflow),
    ];
    for (index, (title, value)) in rows.iter().enumerate() {
        draw_value(
            title,
            &format_money(*value),
            rect.x + 16.0,
            rect.y + 276.0 + index as f32 * 34.0,
            rect.w - 32.0,
        );
    }
    let pressure = 100.0 - auction.seconds_remaining / AUCTION_DURATION_SECONDS * 100.0;
    draw_meter(
        if auction.has_started {
            auction.temperature.label()
        } else {
            "Terms & Registration"
        },
        pressure as i32,
        Rect::new(rect.x + 16.0, rect.y + rect.h - 58.0, rect.w - 32.0, 12.0),
        if auction.has_started {
            temperature_color(auction.temperature)
        } else {
            crate::ui::BLUE
        },
    );
    label_fit(
        if auction.has_started {
            auction.temperature.description()
        } else {
            "The clock waits until you tap START AUCTION CALLS."
        },
        rect.x + 16.0,
        rect.y + rect.h - 18.0,
        rect.w - 32.0,
        14,
        TEXT_DIM,
    );
    if auction.on_market_announced {
        draw_badge(
            "ON MARKET",
            Rect::new(rect.x + rect.w - 214.0, rect.y + 18.0, 100.0, 28.0),
            POSITIVE,
        );
    } else if auction.has_started {
        draw_badge(
            "NOT YET SELLING",
            Rect::new(rect.x + rect.w - 246.0, rect.y + 18.0, 132.0, 28.0),
            WARNING,
        );
    }
}
