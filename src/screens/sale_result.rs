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

        let rect = Rect::new(70.0, 86.0, ui_width() - 140.0, ui_height() - 142.0);
        soft_panel(rect);
        let sold = result.sale_price.is_some();
        label(
            if sold {
                "SOLD UNDER THE HAMMER"
            } else {
                "PASSED IN"
            },
            rect.x + 30.0,
            rect.y + 28.0,
            15,
            if sold { POSITIVE } else { WARNING },
        );
        label(
            if sold {
                "Sale Settled"
            } else {
                "Reserve Protected"
            },
            rect.x + 28.0,
            rect.y + 64.0,
            32,
            TEXT_BRIGHT,
        );
        label(
            &result.property_address,
            rect.x + 30.0,
            rect.y + 92.0,
            19,
            TEXT_DIM,
        );
        let stat_y = rect.y + 116.0;
        let stat_gap = 14.0;
        let stat_w = (rect.w - 60.0 - stat_gap * 2.0) / 3.0;
        let sale_value = result.sale_price.unwrap_or(result.highest_bid);
        let stats = [
            (
                if sold { "Sale Price" } else { "Highest Bid" },
                format_money(sale_value),
                format!("Reserve {}", format_money(result.reserve_price)),
                if sold { ACCENT } else { WARNING },
            ),
            (
                "Settlement Release",
                format_money(result.settlement_release),
                if sold {
                    format!("After {} loan + fees", format_money(result.debt_repaid))
                } else {
                    "No debt cleared".to_string()
                },
                if sold { POSITIVE } else { TEXT_DIM },
            ),
            (
                "Deal Result",
                format_money(result.profit),
                result_verdict(result.profit).to_string(),
                result_color(result.profit),
            ),
        ];
        for (index, (title, value, note, color)) in stats.iter().enumerate() {
            draw_money_stat(
                title,
                value,
                note,
                Rect::new(
                    rect.x + 30.0 + index as f32 * (stat_w + stat_gap),
                    stat_y,
                    stat_w,
                    92.0,
                ),
                *color,
            );
        }

        let room = Rect::new(rect.x + 30.0, rect.y + 220.0, rect.w - 60.0, 58.0);
        dark_panel(room);
        label(
            &format!(
                "{} reserve  {}  |  {} campaign {}  |  {} bidders  |  demand {}/100  |  total deal costs {}",
                result.reserve_choice.label(),
                format_money(result.reserve_price),
                result.marketing_plan.label(),
                format_money(result.marketing_cost),
                result.bidder_count,
                result.demand_score,
                format_money(result.total_costs)
            ),
            room.x + 18.0,
            room.y + 34.0,
            16,
            TEXT,
        );

        let breakdown = Rect::new(rect.x + 30.0, rect.y + 292.0, rect.w - 60.0, 146.0);
        dark_panel(breakdown);
        label(
            "Deal Autopsy",
            breakdown.x + 18.0,
            breakdown.y + 28.0,
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
        let column_w = (breakdown.w - 54.0) / 2.0;
        for (index, (title, value)) in rows.iter().enumerate() {
            let column = index % 2;
            let row = index / 2;
            let x = breakdown.x + 18.0 + column as f32 * (column_w + 18.0);
            let y = breakdown.y + 58.0 + row as f32 * 27.0;
            label(title, x, y, 15, TEXT_DIM);
            label_fit(
                value,
                x + 146.0,
                y,
                column_w - 154.0,
                15,
                autopsy_color(value),
            );
        }

        let lesson = Rect::new(rect.x + 30.0, rect.y + 450.0, rect.w - 60.0, 96.0);
        soft_panel(lesson);
        let half = (lesson.w - 54.0) / 2.0;
        label("Lesson", lesson.x + 18.0, lesson.y + 28.0, 20, TEXT_BRIGHT);
        draw_wrapped_text(
            &result.lesson,
            lesson.x + 18.0,
            lesson.y + 54.0,
            half,
            15,
            TEXT,
        );
        label(
            "Next Auction",
            lesson.x + half + 36.0,
            lesson.y + 28.0,
            20,
            ACCENT,
        );
        draw_wrapped_text(
            &result.next_time,
            lesson.x + half + 36.0,
            lesson.y + 54.0,
            half,
            15,
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
