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

        let rect = Rect::new(120.0, 86.0, ui_width() - 240.0, ui_height() - 126.0);
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
                "{} reserve at {} | {} campaign | {} bidders",
                result.reserve_choice.label(),
                format_money(result.reserve_price),
                result.marketing_plan.label(),
                result.bidder_count
            ),
            rect.x + 30.0,
            rect.y + 98.0,
            15,
            TEXT_DIM,
        );
        label(
            &format!(
                "Highest bid {} | Demand {} / 100 | Cost base {}",
                format_money(result.highest_bid),
                result.demand_score,
                format_money(result.total_costs)
            ),
            rect.x + 30.0,
            rect.y + 118.0,
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
            rect.y + 148.0,
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
            rect.y + 198.0,
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
                rect.y + 232.0,
                22,
                NEGATIVE,
            );
        } else {
            draw_badge(
                result_verdict(result.profit),
                Rect::new(rect.x + 30.0, rect.y + 216.0, 128.0, 28.0),
                result_color(result.profit),
            );
        }

        let breakdown = Rect::new(rect.x + 30.0, rect.y + 260.0, rect.w - 60.0, 194.0);
        dark_panel(breakdown);
        label(
            "Deal Autopsy",
            breakdown.x + 18.0,
            breakdown.y + 30.0,
            22,
            TEXT_BRIGHT,
        );
        let marketing_choice = if result.marketing_choice.is_empty() {
            "Not recorded"
        } else {
            result.marketing_choice.as_str()
        };
        let rows = [
            ("Purchase discipline", result.purchase_discipline.as_str()),
            ("Research quality", result.research_quality.as_str()),
            ("Renovation choice", result.renovation_choice.as_str()),
            ("Marketing", marketing_choice),
            ("Sale timing", result.sale_timing.as_str()),
            ("Reputation", result.reputation_reason.as_str()),
        ];
        for (index, (title, value)) in rows.iter().enumerate() {
            let y = breakdown.y + 58.0 + index as f32 * 22.0;
            label(title, breakdown.x + 18.0, y, 16, TEXT_DIM);
            label_fit(
                value,
                breakdown.x + 220.0,
                y,
                breakdown.w - 238.0,
                16,
                autopsy_color(value),
            );
        }

        let lesson = Rect::new(rect.x + 30.0, rect.y + 472.0, rect.w - 60.0, 96.0);
        soft_panel(lesson);
        label("Lesson", lesson.x + 18.0, lesson.y + 30.0, 22, TEXT_BRIGHT);
        let next_y = draw_wrapped_text(
            &result.lesson,
            lesson.x + 114.0,
            lesson.y + 30.0,
            lesson.w - 132.0,
            16,
            TEXT,
        );
        label("Next", lesson.x + 18.0, next_y + 4.0, 16, ACCENT);
        draw_wrapped_text(
            &result.next_time,
            lesson.x + 114.0,
            next_y + 4.0,
            lesson.w - 132.0,
            16,
            ACCENT,
        );

        let continue_label = if result.sale_price.is_some() {
            "Next Week"
        } else {
            "Back To Portfolio"
        };
        if button(
            Rect::new(rect.x + rect.w - 208.0, rect.y + 24.0, 178.0, 40.0),
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

fn autopsy_color(text: &str) -> Color {
    if text.starts_with("Failed")
        || text.starts_with("Weak")
        || text.starts_with("Wrong tool")
        || text.starts_with("Poor")
        || text.starts_with("Late")
        || text.starts_with("Stretched")
        || text.starts_with("-")
    {
        NEGATIVE
    } else if text.starts_with("Thin")
        || text.starts_with("Partial")
        || text.starts_with("Risky")
        || text.starts_with("Neutral")
        || text.starts_with("No reputation")
    {
        WARNING
    } else {
        POSITIVE
    }
}
