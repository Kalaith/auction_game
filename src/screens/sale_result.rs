use crate::app::App;
use crate::screens::Screen;
use crate::ui::*;
use macroquad::prelude::*;

impl App {
    pub(crate) fn draw_sale_result(&mut self) {
        let Some(result) = self.sale_result.clone() else {
            self.screen = Screen::Dashboard;
            return;
        };

        let rect = Rect::new(190.0, 104.0, screen_width() - 380.0, 540.0);
        panel(rect);
        label(
            "Sale Auction Result",
            rect.x + 28.0,
            rect.y + 44.0,
            31,
            TEXT_BRIGHT,
        );
        label(
            &result.property_address,
            rect.x + 30.0,
            rect.y + 78.0,
            20,
            TEXT_DIM,
        );

        let sold_text = result
            .sale_price
            .map(|price| format!("Sold for {}", format_money(price)))
            .unwrap_or_else(|| "Passed in below reserve".to_string());
        label(
            &sold_text,
            rect.x + 30.0,
            rect.y + 132.0,
            30,
            if result.sale_price.is_some() {
                ACCENT
            } else {
                WARNING
            },
        );

        draw_value(
            "Reserve choice",
            result.reserve_choice.label(),
            rect.x + 30.0,
            rect.y + 178.0,
            rect.w - 60.0,
        );
        draw_value(
            "Reserve",
            &format_money(result.reserve_price),
            rect.x + 30.0,
            rect.y + 206.0,
            rect.w - 60.0,
        );
        draw_value(
            "Highest bid",
            &format_money(result.highest_bid),
            rect.x + 30.0,
            rect.y + 234.0,
            rect.w - 60.0,
        );
        draw_value(
            "Bidder count",
            &result.bidder_count.to_string(),
            rect.x + 30.0,
            rect.y + 262.0,
            rect.w - 60.0,
        );
        draw_value(
            "Market heat",
            &format!("{} / 100", result.demand_score),
            rect.x + 30.0,
            rect.y + 290.0,
            rect.w - 60.0,
        );
        draw_value(
            "Total costs",
            &format_money(result.total_costs),
            rect.x + 30.0,
            rect.y + 318.0,
            rect.w - 60.0,
        );
        draw_value(
            "Selling fees",
            &format_money(result.selling_fees),
            rect.x + 30.0,
            rect.y + 346.0,
            rect.w - 60.0,
        );
        draw_value(
            "Profit / loss",
            &format_money(result.profit),
            rect.x + 30.0,
            rect.y + 374.0,
            rect.w - 60.0,
        );

        let lesson = Rect::new(rect.x + 26.0, rect.y + 400.0, rect.w - 52.0, 70.0);
        dark_panel(lesson);
        label("Lesson", lesson.x + 14.0, lesson.y + 26.0, 20, TEXT_BRIGHT);
        draw_wrapped_text(
            &result.lesson,
            lesson.x + 104.0,
            lesson.y + 26.0,
            lesson.w - 118.0,
            18,
            TEXT,
        );

        let continue_label = if result.sale_price.is_some() {
            "Next Deal"
        } else {
            "Back To Portfolio"
        };
        if button(
            Rect::new(rect.x + rect.w - 208.0, rect.y + rect.h - 58.0, 178.0, 40.0),
            continue_label,
            true,
            ButtonTone::Primary,
        ) {
            if result.sale_price.is_some() {
                self.advance_week();
                self.screen = Screen::Dashboard;
            } else {
                self.screen = Screen::Portfolio;
            }
        }
    }
}
