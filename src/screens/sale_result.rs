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

        let rect = Rect::new(190.0, 104.0, ui_width() - 380.0, ui_height() - 154.0);
        soft_panel(rect);
        label("Sale Result", rect.x + 28.0, rect.y + 44.0, 31, TEXT_BRIGHT);
        label(
            &result.property_address,
            rect.x + 30.0,
            rect.y + 76.0,
            19,
            TEXT_DIM,
        );
        label(
            &format!(
                "{} reserve at {} | {} bidders",
                result.reserve_choice.label(),
                format_money(result.reserve_price),
                result.bidder_count
            ),
            rect.x + 30.0,
            rect.y + 98.0,
            15,
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
            35,
            if result.sale_price.is_some() {
                ACCENT
            } else {
                WARNING
            },
        );
        label(
            &format!("Result {}", format_money(result.profit)),
            rect.x + 30.0,
            rect.y + 182.0,
            42,
            if result.profit >= 0 {
                POSITIVE
            } else {
                NEGATIVE
            },
        );
        let over_walkaway = (result.purchase_price - result.walkaway_price).max(0);
        if over_walkaway > 0 {
            label(
                &format!("Over walk-away by {}", format_money(over_walkaway)),
                rect.x + 30.0,
                rect.y + 218.0,
                22,
                NEGATIVE,
            );
        } else {
            draw_badge(
                result_verdict(result.profit),
                Rect::new(rect.x + 30.0, rect.y + 198.0, 128.0, 28.0),
                result_color(result.profit),
            );
        }

        let breakdown = Rect::new(rect.x + 30.0, rect.y + 246.0, rect.w - 60.0, 148.0);
        dark_panel(breakdown);
        label(
            "What Happened",
            breakdown.x + 18.0,
            breakdown.y + 30.0,
            22,
            TEXT_BRIGHT,
        );
        let sale_price = result.sale_price.unwrap_or(result.highest_bid);
        let acquisition_costs = result.total_costs - result.selling_fees;
        let before_fees = sale_price - acquisition_costs;
        let rows = [
            ("Over walk-away", -over_walkaway),
            ("Sale against cost base", before_fees),
            ("Selling fees", -result.selling_fees),
            ("Market heat", i64::from(result.demand_score)),
            ("Final profit / loss", result.profit),
        ];
        for (index, (title, value)) in rows.iter().enumerate() {
            let y = breakdown.y + 56.0 + index as f32 * 20.0;
            label(title, breakdown.x + 18.0, y, 16, TEXT_DIM);
            let value_text = if *title == "Market heat" {
                format!("{value} / 100")
            } else {
                format_money(*value)
            };
            let measured = measure_label(&value_text, 18);
            label(
                &value_text,
                breakdown.x + breakdown.w - measured.width - 18.0,
                y,
                18,
                if *value >= 0 { POSITIVE } else { NEGATIVE },
            );
        }

        let lesson = Rect::new(rect.x + 30.0, rect.y + 414.0, rect.w - 60.0, 78.0);
        soft_panel(lesson);
        label("Lesson", lesson.x + 18.0, lesson.y + 30.0, 22, TEXT_BRIGHT);
        draw_wrapped_text(
            &result.lesson,
            lesson.x + 114.0,
            lesson.y + 30.0,
            lesson.w - 132.0,
            18,
            TEXT,
        );

        let continue_label = if result.sale_price.is_some() {
            "Next Week"
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

fn result_verdict(profit: i64) -> &'static str {
    if profit >= 45_000 {
        "Strong Exit"
    } else if profit >= 0 {
        "Clean Exit"
    } else if profit > -40_000 {
        "Thin Loss"
    } else {
        "Bad Deal"
    }
}

fn result_color(profit: i64) -> Color {
    if profit >= 0 {
        POSITIVE
    } else if profit > -40_000 {
        WARNING
    } else {
        NEGATIVE
    }
}
