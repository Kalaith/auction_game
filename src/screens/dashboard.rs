use crate::app::App;
use crate::model::{Property, CAMPAIGN_GOAL_NET_WORTH};
use crate::screens::Screen;
use crate::sim::campaign::next_unlock_note;
use crate::sim::finance::max_financeable_bid;
use crate::sim::valuation::{estimated_value_range, net_worth};
use crate::ui::*;
use macroquad::prelude::*;

impl App {
    pub(crate) fn draw_dashboard(&mut self) {
        let margin = 28.0;
        let top = 92.0;
        let width = ui_width() - margin * 2.0;
        let current_net_worth = net_worth(&self.player, self.market());

        self.draw_dashboard_stats(margin, top, width, current_net_worth);
        self.draw_market_pulse(margin, top + 98.0, width, current_net_worth);

        label(
            "Featured Opportunities",
            margin,
            top + 288.0,
            25,
            TEXT_BRIGHT,
        );
        label(
            "Choose the auction worth your attention.",
            margin + 330.0,
            top + 288.0,
            16,
            TEXT_DIM,
        );

        let card_w = (width - 36.0) / 3.0;
        let mut action = None;
        for (slot, property) in self.available_properties.iter().take(3).enumerate() {
            let x = margin + slot as f32 * (card_w + 18.0);
            let rect = Rect::new(x, top + 314.0, card_w, 164.0);
            if slot == 0 {
                highlight_panel(rect);
            } else {
                soft_panel(rect);
            }
            if dashboard_property_card(self, rect, property) {
                action = Some(slot);
            }
        }

        let action_y = ui_height() - 88.0;
        if button(
            Rect::new(ui_width() - 382.0, action_y, 168.0, 40.0),
            "Advance Week",
            !self.campaign_status.is_finished(),
            ButtonTone::Ghost,
        ) {
            self.advance_week();
        }
        if button(
            Rect::new(ui_width() - 196.0, action_y, 168.0, 40.0),
            "See Listings",
            true,
            ButtonTone::Secondary,
        ) {
            self.screen = Screen::PropertyList;
        }
        if let Some(index) = action {
            self.open_property_detail(index);
        }
    }

    fn draw_dashboard_stats(&self, x: f32, y: f32, width: f32, net_worth_value: i64) {
        let gap = 12.0;
        let card_w = (width - gap * 3.0) / 4.0;
        let progress = net_worth_value as f32 / CAMPAIGN_GOAL_NET_WORTH as f32;
        let buying_power = max_financeable_bid(&self.player, self.market());
        let stats = [
            (
                "Cash",
                format_compact_money(self.player.cash),
                "Ready capital",
                POSITIVE,
            ),
            (
                "Buying Power",
                format_compact_money(buying_power),
                "Max next bid",
                POSITIVE,
            ),
            (
                "Goal",
                format!("{:.0}%", progress.clamp(0.0, 1.0) * 100.0),
                "To $1.0m",
                ACCENT,
            ),
            (
                "Net Worth",
                format_compact_money(net_worth_value),
                "After debt",
                crate::ui::BLUE,
            ),
        ];

        for (index, (title, value, note, color)) in stats.iter().enumerate() {
            let rect = Rect::new(x + index as f32 * (card_w + gap), y, card_w, 78.0);
            draw_money_stat(title, value, note, rect, *color);
        }
    }

    fn draw_market_pulse(&self, x: f32, y: f32, width: f32, net_worth_value: i64) {
        let rect = Rect::new(x, y, width, 158.0);
        soft_panel(rect);
        label(
            "Market Pulse",
            rect.x + 18.0,
            rect.y + 32.0,
            25,
            TEXT_BRIGHT,
        );
        label(
            &self.market().title,
            rect.x + 210.0,
            rect.y + 32.0,
            15,
            TEXT_DIM,
        );
        draw_badge(
            "THIS WEEK",
            Rect::new(rect.x + rect.w - 118.0, rect.y + 17.0, 92.0, 26.0),
            crate::ui::BLUE,
        );

        for (index, item) in self.market().items.iter().take(2).enumerate() {
            let y = rect.y + 64.0 + index as f32 * 30.0;
            let (headline, _) = split_market_line(item);
            label_fit(&headline, rect.x + 28.0, y, rect.w - 190.0, 18, TEXT_BRIGHT);
        }

        let unlock = Rect::new(rect.x + 18.0, rect.y + rect.h - 36.0, rect.w - 36.0, 24.0);
        draw_rectangle(
            unlock.x,
            unlock.y,
            unlock.w,
            unlock.h,
            Color::new(ACCENT.r * 0.20, ACCENT.g * 0.16, ACCENT.b * 0.08, 1.0),
        );
        label("NEXT", unlock.x + 12.0, unlock.y + 17.0, 14, ACCENT);
        label(
            next_unlock_note(self.week, net_worth_value, self.player.reputation),
            unlock.x + 70.0,
            unlock.y + 17.0,
            14,
            TEXT_DIM,
        );
    }
}

fn dashboard_property_card(app: &App, rect: Rect, property: &Property) -> bool {
    draw_house_art(
        Rect::new(rect.x + 14.0, rect.y + 18.0, 146.0, 94.0),
        property,
    );
    let text_x = rect.x + 178.0;
    let text_w = rect.w - 198.0;
    label_fit(
        &property.address,
        text_x,
        rect.y + 38.0,
        text_w,
        21,
        TEXT_BRIGHT,
    );
    label(&property.suburb, text_x, rect.y + 66.0, 16, TEXT_DIM);
    label(
        reason_to_care(property, app),
        text_x,
        rect.y + 90.0,
        16,
        verdict_color(property, app),
    );
    label(
        &format!("Guide {}", format_money(property.guide_price)),
        text_x,
        rect.y + 116.0,
        20,
        POSITIVE,
    );
    let pressed = button(
        Rect::new(text_x, rect.y + 130.0, text_w, 30.0),
        "View Details",
        true,
        ButtonTone::Primary,
    );
    pressed || rect_clicked(rect)
}

fn split_market_line(item: &str) -> (String, String) {
    let Some((headline, rest)) = item.split_once('.') else {
        return (item.to_string(), String::new());
    };
    (headline.to_string(), rest.trim().to_string())
}

fn reason_to_care(property: &Property, app: &App) -> &'static str {
    if property.hidden_defect_risk >= 0.28 {
        "Risky fixer"
    } else if upside_amount(property, app) >= 95_000 {
        "High upside"
    } else if property.buyer_demand >= 70 {
        "Family demand"
    } else if property.guide_price <= 500_000 {
        "Cheap entry"
    } else {
        "Steady play"
    }
}

fn verdict_color(property: &Property, app: &App) -> Color {
    if property.hidden_defect_risk >= 0.28 {
        WARNING
    } else if upside_amount(property, app) >= 95_000 {
        POSITIVE
    } else {
        TEXT_DIM
    }
}

fn upside_amount(property: &Property, app: &App) -> i64 {
    let (_, high) = estimated_value_range(property, app.market());
    high - property.guide_price
}
