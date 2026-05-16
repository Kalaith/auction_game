use crate::app::App;
use crate::model::{Property, ResearchLevel};
use crate::screens::Screen;
use crate::sim::finance::finance_snapshot;
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

        label("Property Decision", 28.0, 106.0, 18, crate::ui::BLUE);
        label(&property.address, 28.0, 136.0, 32, TEXT_BRIGHT);
        label(
            &format!(
                "{} | {} bed, {} bath | {}sqm | {}",
                property.suburb,
                property.bedrooms,
                property.bathrooms,
                property.land_size,
                property.condition.label()
            ),
            30.0,
            164.0,
            17,
            TEXT_DIM,
        );

        let hero = Rect::new(28.0, 188.0, 500.0, 286.0);
        soft_panel(hero);
        draw_house_art(
            Rect::new(hero.x + 14.0, hero.y + 14.0, hero.w - 28.0, 206.0),
            &property,
        );
        draw_badge(
            property.condition.label().to_uppercase().as_str(),
            Rect::new(hero.x + 18.0, hero.y + 236.0, 92.0, 26.0),
            condition_color(&property),
        );
        draw_badge(
            risk_badge(&property),
            Rect::new(hero.x + 120.0, hero.y + 236.0, 102.0, 26.0),
            risk_color(&property, research_level),
        );
        draw_badge(
            research_level.confidence_label(),
            Rect::new(hero.x + 232.0, hero.y + 236.0, 128.0, 26.0),
            crate::ui::BLUE,
        );

        let decision = Rect::new(552.0, 122.0, ui_width() - 580.0, 352.0);
        soft_panel(decision);
        draw_detail_summary(self, decision, &property, research_level);

        for (button_index, level) in research_level.next_levels().iter().take(3).enumerate() {
            let button_rect = Rect::new(
                decision.x + 22.0 + button_index as f32 * 126.0,
                decision.y + decision.h - 46.0,
                114.0,
                30.0,
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

        let walk = Rect::new(28.0, ui_height() - 142.0, ui_width() - 56.0, 92.0);
        soft_panel(walk);
        draw_walkaway_panel(self, walk, &property);

        if let Some(level) = research_action {
            self.buy_research(property.id, level);
        }
    }
}

fn draw_detail_summary(app: &App, rect: Rect, property: &Property, research_level: ResearchLevel) {
    let (low, high) = researched_value_range(property, app.market(), research_level);
    let walkaway = recommended_walkaway(property, app.market(), research_level);
    let margin = projected_purchase_margin(property, walkaway, app.market());

    draw_badge(
        upside_badge(property),
        Rect::new(rect.x + 22.0, rect.y + 20.0, 128.0, 28.0),
        POSITIVE,
    );
    draw_badge(
        demand_badge(property),
        Rect::new(rect.x + 162.0, rect.y + 20.0, 118.0, 28.0),
        crate::ui::BLUE,
    );

    label("Guide Price", rect.x + 22.0, rect.y + 86.0, 16, TEXT_DIM);
    label(
        &format_money(property.guide_price),
        rect.x + 22.0,
        rect.y + 120.0,
        34,
        ACCENT,
    );

    label(
        "Research Range",
        rect.x + 22.0,
        rect.y + 158.0,
        16,
        TEXT_DIM,
    );
    label(
        &format!("{} - {}", format_money(low), format_money(high)),
        rect.x + 22.0,
        rect.y + 186.0,
        23,
        TEXT_BRIGHT,
    );
    label(
        &format!(
            "Nearest comp: {}",
            format_money(comparable_sale_value(property, app.market(), 0))
        ),
        rect.x + 22.0,
        rect.y + 206.0,
        14,
        TEXT_DIM,
    );

    label(
        "Recommended Walk-away",
        rect.x + 22.0,
        rect.y + 226.0,
        16,
        TEXT_DIM,
    );
    label(
        &format_money(walkaway),
        rect.x + 22.0,
        rect.y + 258.0,
        30,
        if margin >= 0 { POSITIVE } else { WARNING },
    );
    label(
        &format!("Projected margin: {}", format_money(margin)),
        rect.x + 22.0,
        rect.y + 286.0,
        16,
        if margin >= 0 { POSITIVE } else { WARNING },
    );

    let risk = Rect::new(rect.x + rect.w - 286.0, rect.y + 74.0, 260.0, 190.0);
    dark_panel(risk);
    label("Risk Read", risk.x + 16.0, risk.y + 30.0, 21, TEXT_BRIGHT);
    let mut y = draw_wrapped_text(
        &risk_summary(property, research_level),
        risk.x + 16.0,
        risk.y + 62.0,
        risk.w - 32.0,
        16,
        risk_color(property, research_level),
    );
    y += 4.0;
    y = draw_wrapped_text(
        due_diligence_note(property, research_level),
        risk.x + 16.0,
        y,
        risk.w - 32.0,
        15,
        TEXT_DIM,
    );
    if y < risk.y + risk.h - 18.0 {
        label_fit(
            &property.notes,
            risk.x + 16.0,
            y + 2.0,
            risk.w - 32.0,
            13,
            TEXT_DIM,
        );
    }
}

fn draw_walkaway_panel(app: &mut App, rect: Rect, property: &Property) {
    let margin = projected_purchase_margin(property, app.walkaway_price, app.market());
    let finance = finance_snapshot(&app.player, app.market(), app.walkaway_price);
    label(
        "Walk-away Price",
        rect.x + 18.0,
        rect.y + 30.0,
        23,
        TEXT_BRIGHT,
    );
    label(
        "This is your line in the sand.",
        rect.x + 18.0,
        rect.y + 56.0,
        16,
        TEXT_DIM,
    );
    label(
        &format_money(app.walkaway_price),
        rect.x + 292.0,
        rect.y + 52.0,
        34,
        if margin >= 0 { POSITIVE } else { WARNING },
    );
    label(
        walkaway_verdict(margin),
        rect.x + 512.0,
        rect.y + 48.0,
        19,
        if margin >= 0 { POSITIVE } else { WARNING },
    );
    label(
        &format!(
            "Cash if won: {} | Margin: {} | Bank room: {}",
            format_money(cash_needed_to_settle(app.walkaway_price)),
            format_money(margin),
            format_money(finance.headroom_after)
        ),
        rect.x + 292.0,
        rect.y + 76.0,
        16,
        TEXT_DIM,
    );
    draw_badge(
        finance.stress.label(),
        Rect::new(rect.x + 640.0, rect.y + 26.0, 112.0, 24.0),
        finance_color(finance.stress),
    );

    if button(
        Rect::new(rect.x + rect.w - 392.0, rect.y + 24.0, 74.0, 40.0),
        "-10k",
        true,
        ButtonTone::Ghost,
    ) {
        app.walkaway_price = (app.walkaway_price - 10_000).max(property.guide_price - 80_000);
    }
    if button(
        Rect::new(rect.x + rect.w - 308.0, rect.y + 24.0, 74.0, 40.0),
        "+10k",
        true,
        ButtonTone::Ghost,
    ) {
        app.walkaway_price += 10_000;
    }
    if button(
        Rect::new(rect.x + rect.w - 220.0, rect.y + 18.0, 190.0, 52.0),
        "Register To Bid",
        true,
        ButtonTone::Primary,
    ) {
        app.start_auction(property.id);
    }
}

fn finance_color(stress: crate::sim::finance::FinanceStress) -> Color {
    match stress {
        crate::sim::finance::FinanceStress::Healthy => POSITIVE,
        crate::sim::finance::FinanceStress::Tight => WARNING,
        crate::sim::finance::FinanceStress::Maxed => NEGATIVE,
    }
}

fn walkaway_verdict(margin: i64) -> &'static str {
    if margin >= 45_000 {
        "Safe bid plan"
    } else if margin >= 0 {
        "Thin margin"
    } else {
        "Bad deal line"
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

fn upside_badge(property: &Property) -> &'static str {
    if property.renovation_potential >= 80 {
        "STRONG UPSIDE"
    } else if property.renovation_potential >= 60 {
        "GOOD UPSIDE"
    } else {
        "LOW UPSIDE"
    }
}

fn demand_badge(property: &Property) -> &'static str {
    if property.buyer_demand >= 70 {
        "HOT DEMAND"
    } else if property.buyer_demand >= 55 {
        "STEADY DEMAND"
    } else {
        "SOFT DEMAND"
    }
}

fn risk_badge(property: &Property) -> &'static str {
    if property.hidden_defect_risk >= 0.28 {
        "HIGH RISK"
    } else if property.hidden_defect_risk >= 0.16 {
        "UNKNOWN"
    } else {
        "LOW RISK"
    }
}

fn risk_color(property: &Property, level: ResearchLevel) -> Color {
    if level >= ResearchLevel::BuildingInspection
        && crate::sim::research::material_defect_likely(property)
    {
        NEGATIVE
    } else if property.hidden_defect_risk >= 0.18 {
        WARNING
    } else {
        POSITIVE
    }
}

fn condition_color(property: &Property) -> Color {
    match property.condition {
        crate::model::Condition::Rough => NEGATIVE,
        crate::model::Condition::Tired => WARNING,
        crate::model::Condition::Solid => POSITIVE,
        crate::model::Condition::Premium => crate::ui::BLUE,
    }
}
