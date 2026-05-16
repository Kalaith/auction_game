use crate::app::App;
use crate::model::ResearchLevel;
use crate::screens::Screen;
use crate::sim::research::{
    comparable_sale_value, due_diligence_note, recommended_walkaway, researched_value_range,
    risk_summary,
};
use crate::sim::valuation::{cash_needed_to_settle, projected_purchase_margin};
use crate::ui::*;
use macroquad::prelude::*;

impl App {
    pub(crate) fn draw_property_detail(&mut self, index: usize) {
        let Some(property) = self.available_properties.get(index).cloned() else {
            self.screen = Screen::PropertyList;
            return;
        };
        let research_level = self.research_level(property.id);
        let mut research_action = None;

        let main = Rect::new(28.0, 92.0, screen_width() - 56.0, screen_height() - 142.0);
        panel(main);
        draw_house_art(
            Rect::new(main.x + 18.0, main.y + 18.0, 360.0, 230.0),
            &property,
        );

        label(
            &property.address,
            main.x + 400.0,
            main.y + 44.0,
            31,
            TEXT_BRIGHT,
        );
        label(
            &format!(
                "{} | {} bed, {} bath | {}sqm | {}",
                property.suburb,
                property.bedrooms,
                property.bathrooms,
                property.land_size,
                property.condition.label()
            ),
            main.x + 402.0,
            main.y + 76.0,
            18,
            TEXT_DIM,
        );

        let (low, high) = researched_value_range(&property, self.market(), research_level);
        let stats = Rect::new(main.x + 402.0, main.y + 104.0, 390.0, 164.0);
        dark_panel(stats);
        draw_value(
            "Guide price",
            &format_money(property.guide_price),
            stats.x + 16.0,
            stats.y + 38.0,
            stats.w - 32.0,
        );
        draw_value(
            "Reserve estimate",
            &format_money(property.reserve_price),
            stats.x + 16.0,
            stats.y + 74.0,
            stats.w - 32.0,
        );
        draw_value(
            "Research range",
            &format!("{} - {}", format_money(low), format_money(high)),
            stats.x + 16.0,
            stats.y + 110.0,
            stats.w - 32.0,
        );
        draw_value(
            "Confidence",
            research_level.confidence_label(),
            stats.x + 16.0,
            stats.y + 146.0,
            stats.w - 32.0,
        );

        let report = Rect::new(main.x + 820.0, main.y + 104.0, main.w - 846.0, 164.0);
        dark_panel(report);
        label(
            "Inspection Notes",
            report.x + 16.0,
            report.y + 30.0,
            22,
            TEXT_BRIGHT,
        );
        draw_wrapped_text(
            &property.notes,
            report.x + 16.0,
            report.y + 62.0,
            report.w - 32.0,
            18,
            TEXT,
        );

        let compare = Rect::new(main.x + 18.0, main.y + 280.0, 590.0, 170.0);
        dark_panel(compare);
        label(
            "Comparable Sales",
            compare.x + 16.0,
            compare.y + 32.0,
            22,
            TEXT_BRIGHT,
        );
        label(
            &format!(
                "{} from {}",
                research_level.confidence_label(),
                research_level.label()
            ),
            compare.x + 196.0,
            compare.y + 32.0,
            16,
            TEXT_DIM,
        );
        for row in 0..3 {
            let comp_value = comparable_sale_value(&property, self.market(), row);
            label(
                &format!(
                    "{} comparable #{} sold for {}",
                    property.suburb,
                    row + 1,
                    format_money(comp_value)
                ),
                compare.x + 18.0,
                compare.y + 70.0 + row as f32 * 30.0,
                18,
                TEXT,
            );
        }

        let research = Rect::new(main.x + 626.0, main.y + 280.0, main.w - 644.0, 170.0);
        dark_panel(research);
        label(
            "Research",
            research.x + 16.0,
            research.y + 30.0,
            22,
            TEXT_BRIGHT,
        );
        label(
            &format!("Current: {}", research_level.label()),
            research.x + 150.0,
            research.y + 30.0,
            16,
            TEXT_DIM,
        );
        draw_wrapped_text(
            &risk_summary(&property, research_level),
            research.x + 16.0,
            research.y + 58.0,
            research.w - 32.0,
            16,
            risk_color(&property, research_level),
        );
        draw_wrapped_text(
            due_diligence_note(&property, research_level),
            research.x + 16.0,
            research.y + 88.0,
            research.w - 32.0,
            15,
            TEXT_DIM,
        );
        label(
            &format!("Trend: {}", suburb_trend_line(self, &property.suburb)),
            research.x + 16.0,
            research.y + 118.0,
            15,
            TEXT,
        );
        for (button_index, level) in research_level.next_levels().iter().take(3).enumerate() {
            let button_rect = Rect::new(
                research.x + 16.0 + button_index as f32 * 126.0,
                research.y + 134.0,
                114.0,
                28.0,
            );
            if button(
                button_rect,
                research_button_label(*level),
                self.player.cash >= level.cost(),
                ButtonTone::Secondary,
            ) {
                research_action = Some(*level);
            }
        }

        let walk = Rect::new(main.x + 18.0, main.y + 468.0, main.w - 36.0, 98.0);
        dark_panel(walk);
        label(
            "Walk-away reminder",
            walk.x + 16.0,
            walk.y + 32.0,
            21,
            TEXT_BRIGHT,
        );
        label(
            &format!(
                "{} | Cash needed: {} | Margin: {}",
                format_money(self.walkaway_price),
                format_money(cash_needed_to_settle(self.walkaway_price)),
                format_money(projected_purchase_margin(
                    &property,
                    self.walkaway_price,
                    self.market()
                ))
            ),
            walk.x + 16.0,
            walk.y + 60.0,
            18,
            TEXT_DIM,
        );
        label(
            &format!(
                "Research recommended walk-away: {}",
                format_money(recommended_walkaway(
                    &property,
                    self.market(),
                    research_level
                ))
            ),
            walk.x + 16.0,
            walk.y + 78.0,
            16,
            TEXT_DIM,
        );

        if button(
            Rect::new(walk.x + walk.w - 390.0, walk.y + 20.0, 72.0, 38.0),
            "-10k",
            true,
            ButtonTone::Ghost,
        ) {
            self.walkaway_price = (self.walkaway_price - 10_000).max(property.guide_price - 80_000);
        }
        if button(
            Rect::new(walk.x + walk.w - 306.0, walk.y + 20.0, 72.0, 38.0),
            "+10k",
            true,
            ButtonTone::Ghost,
        ) {
            self.walkaway_price += 10_000;
        }
        if button(
            Rect::new(walk.x + walk.w - 220.0, walk.y + 18.0, 190.0, 42.0),
            "Register To Bid",
            true,
            ButtonTone::Primary,
        ) {
            self.start_auction(property.id);
        }
        if let Some(level) = research_action {
            self.buy_research(property.id, level);
        }
    }
}

fn research_button_label(level: ResearchLevel) -> &'static str {
    match level {
        ResearchLevel::StreetScan => "Street $0",
        ResearchLevel::AgentPack => "Agent $1.5k",
        ResearchLevel::BuildingInspection => "Build $3.5k",
        ResearchLevel::FullDiligence => "Full $6k",
    }
}

fn suburb_trend_line(app: &App, suburb: &str) -> String {
    app.data
        .market_events
        .iter()
        .map(|event| format!("{:+.0}%", event.suburb_modifier(suburb) * 100.0))
        .collect::<Vec<_>>()
        .join(" / ")
}

fn risk_color(property: &crate::model::Property, level: ResearchLevel) -> Color {
    if level >= ResearchLevel::BuildingInspection
        && crate::sim::research::material_defect_likely(property)
    {
        NEGATIVE
    } else if property.hidden_defect_risk >= 0.18 {
        WARNING
    } else {
        TEXT
    }
}
