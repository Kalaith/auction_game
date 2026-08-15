use crate::app::App;
use crate::model::{Property, ResearchLevel, WalkawayStyle};
use crate::screens::Screen;
use crate::sim::finance::finance_snapshot;
use crate::sim::rental::weekly_rent_for;
use crate::sim::research::{
    comparable_sale_value, due_diligence_note, estimate_reserve, recommended_walkaway,
    research_cost, research_fit_summary, research_question, research_takeaway,
    researched_value_range, risk_summary,
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
        label_fit(
            &property.notes,
            hero.x + 18.0,
            hero.y + 276.0,
            hero.w - 36.0,
            13,
            TEXT_DIM,
        );

        let decision = Rect::new(552.0, 122.0, ui_width() - 580.0, 416.0);
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
                &research_button_label(*level, self.player.reputation),
                self.player.cash >= research_cost(*level, self.player.reputation),
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
    let (low, high) = researched_value_range(
        property,
        app.market(),
        research_level,
        app.player.reputation,
    );
    let walkaway = recommended_walkaway(
        property,
        app.market(),
        research_level,
        app.walkaway_style,
        app.player.reputation,
    );
    let margin = projected_purchase_margin(property, walkaway, app.market());
    let weekly_rent = weekly_rent_for(property, app.market());
    let gross_yield = weekly_rent as f32 * 52.0 / walkaway.max(1) as f32 * 100.0;

    draw_badge(
        property.deal_archetype.label(),
        Rect::new(rect.x + 22.0, rect.y + 20.0, 128.0, 28.0),
        ACCENT,
    );
    draw_badge(
        upside_badge(property),
        Rect::new(rect.x + 162.0, rect.y + 20.0, 118.0, 28.0),
        POSITIVE,
    );
    draw_badge(
        demand_badge(property),
        Rect::new(rect.x + 292.0, rect.y + 20.0, 118.0, 28.0),
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
    label_fit(
        &format!(
            "Comp {}  |  Reserve read {}",
            format_money(comparable_sale_value(property, app.market(), 0)),
            format_money(estimate_reserve(
                property,
                app.market(),
                research_level,
                app.player.reputation
            ))
        ),
        rect.x + 22.0,
        rect.y + 206.0,
        330.0,
        14,
        TEXT_DIM,
    );
    label_fit(
        &format!(
            "Rent {} / wk  |  {:.1}% gross at walk-away",
            format_money(weekly_rent),
            gross_yield
        ),
        rect.x + 22.0,
        rect.y + 226.0,
        330.0,
        14,
        if gross_yield >= 5.0 {
            POSITIVE
        } else {
            WARNING
        },
    );

    label(
        "Recommended Walk-away",
        rect.x + 22.0,
        rect.y + 250.0,
        16,
        TEXT_DIM,
    );
    label(
        &format_money(walkaway),
        rect.x + 22.0,
        rect.y + 282.0,
        30,
        if margin >= 0 { POSITIVE } else { WARNING },
    );
    label(
        &format!(
            "{} plan | Projected margin: {}",
            app.walkaway_style.label(),
            format_money(margin)
        ),
        rect.x + 22.0,
        rect.y + 308.0,
        16,
        if margin >= 0 { POSITIVE } else { WARNING },
    );

    let thesis = Rect::new(rect.x + rect.w - 286.0, rect.y + 62.0, 260.0, 118.0);
    dark_panel(thesis);
    label(
        "Deal Thesis",
        thesis.x + 16.0,
        thesis.y + 28.0,
        20,
        TEXT_BRIGHT,
    );
    label_fit(
        &property.thesis,
        thesis.x + 16.0,
        thesis.y + 56.0,
        thesis.w - 32.0,
        15,
        TEXT,
    );
    label_fit(
        &format!("Risk: {}", property.main_risk),
        thesis.x + 16.0,
        thesis.y + 80.0,
        thesis.w - 32.0,
        14,
        WARNING,
    );
    label_fit(
        &format!("Trap: {}", property.bad_strategy),
        thesis.x + 16.0,
        thesis.y + 102.0,
        thesis.w - 32.0,
        14,
        TEXT_DIM,
    );

    let risk = Rect::new(rect.x + rect.w - 286.0, rect.y + 194.0, 260.0, 150.0);
    dark_panel(risk);
    label(
        "Due Diligence",
        risk.x + 16.0,
        risk.y + 28.0,
        20,
        TEXT_BRIGHT,
    );
    label_fit(
        research_question(research_level),
        risk.x + 16.0,
        risk.y + 54.0,
        risk.w - 32.0,
        14,
        crate::ui::BLUE,
    );
    label_fit(
        research_fit_summary(property, research_level),
        risk.x + 16.0,
        risk.y + 74.0,
        risk.w - 32.0,
        14,
        risk_color(property, research_level),
    );
    let takeaway = format!(
        "{} {} {}",
        risk_summary(property, research_level),
        research_takeaway(property, app.market(), research_level),
        due_diligence_note(property, research_level)
    );
    label_fit(
        &takeaway,
        risk.x + 16.0,
        risk.y + 94.0,
        risk.w - 32.0,
        13,
        TEXT_DIM,
    );
    let likely_profile = &app.data.bidder_profiles[property.id % app.data.bidder_profiles.len()];
    let rival_hint = if research_level >= ResearchLevel::FullDiligence {
        format!(
            "Likely rival: {} ({})",
            likely_profile.name,
            likely_profile.bidder_type.label()
        )
    } else {
        "Full diligence profiles one likely rival.".to_string()
    };
    label_fit(
        &rival_hint,
        risk.x + 16.0,
        risk.y + 122.0,
        risk.w - 32.0,
        14,
        if research_level >= ResearchLevel::FullDiligence {
            ACCENT
        } else {
            TEXT_DIM
        },
    );
}

fn draw_walkaway_panel(app: &mut App, rect: Rect, property: &Property) {
    let margin = projected_purchase_margin(property, app.walkaway_price, app.market());
    let finance = finance_snapshot(&app.player, app.market(), app.walkaway_price);
    label(
        "Walk-away Strategy",
        rect.x + 18.0,
        rect.y + 30.0,
        23,
        TEXT_BRIGHT,
    );
    let styles = [
        WalkawayStyle::Conservative,
        WalkawayStyle::Balanced,
        WalkawayStyle::Aggressive,
    ];
    for (index, style) in styles.iter().enumerate() {
        let selected = app.walkaway_style == *style;
        if button(
            Rect::new(
                rect.x + 18.0 + index as f32 * 112.0,
                rect.y + 42.0,
                102.0,
                28.0,
            ),
            style.label(),
            true,
            if selected {
                ButtonTone::Primary
            } else {
                ButtonTone::Ghost
            },
        ) {
            app.walkaway_style = *style;
            app.walkaway_price = recommended_walkaway(
                property,
                app.market(),
                app.research_level(property.id),
                app.walkaway_style,
                app.player.reputation,
            );
            app.status = format!(
                "{} walk-away selected: {}",
                style.label(),
                style.description()
            );
        }
    }
    label(
        app.walkaway_style.description(),
        rect.x + 18.0,
        rect.y + 84.0,
        14,
        TEXT_DIM,
    );
    label(
        &format_money(app.walkaway_price),
        rect.x + 386.0,
        rect.y + 52.0,
        34,
        if margin >= 0 { POSITIVE } else { WARNING },
    );
    label(
        walkaway_verdict(margin),
        rect.x + 600.0,
        rect.y + 48.0,
        19,
        if margin >= 0 { POSITIVE } else { WARNING },
    );
    label(
        &format!(
            "Cash to settle: {} | Margin: {} | Bank room: {}",
            format_money(cash_needed_to_settle(app.walkaway_price)),
            format_money(margin),
            format_money(finance.headroom_after)
        ),
        rect.x + 386.0,
        rect.y + 76.0,
        16,
        TEXT_DIM,
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
    let can_register = app.auction_registrations > 0;
    if button(
        Rect::new(rect.x + rect.w - 220.0, rect.y + 18.0, 190.0, 52.0),
        if can_register {
            "REGISTER TO BID"
        } else {
            "VIEW DASHBOARD"
        },
        true,
        if can_register {
            ButtonTone::Primary
        } else {
            ButtonTone::Secondary
        },
    ) {
        if can_register {
            app.start_auction(property.id);
        } else {
            app.screen = Screen::Dashboard;
            app.status = "Tap ADVANCE WEEK to refresh your two registrations.".to_string();
        }
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

fn research_button_label(level: ResearchLevel, reputation: i32) -> String {
    match level {
        ResearchLevel::StreetScan => "Street $0".to_string(),
        ResearchLevel::AgentPack => format!(
            "Agent {}",
            format_compact_money(research_cost(level, reputation))
        ),
        ResearchLevel::BuildingInspection => {
            format!(
                "Build {}",
                format_compact_money(research_cost(level, reputation))
            )
        }
        ResearchLevel::FullDiligence => {
            format!(
                "Full {}",
                format_compact_money(research_cost(level, reputation))
            )
        }
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
