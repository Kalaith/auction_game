use crate::app::App;
use crate::model::Property;
use crate::sim::valuation::estimated_value_range;
use crate::ui::*;
use macroquad::prelude::*;

const FILTERS: [&str; 5] = [
    "All",
    "Low Risk",
    "High Upside",
    "Hot Demand",
    "Cheap Entry",
];

impl App {
    pub(crate) fn draw_property_list(&mut self) {
        label("Auction Listings", 28.0, 106.0, 30, TEXT_BRIGHT);
        label(
            "Pick the auction that deserves your attention this week.",
            30.0,
            134.0,
            18,
            TEXT_DIM,
        );

        for (index, filter) in FILTERS.iter().enumerate() {
            let selected = self.listing_filter == index;
            let tone = if selected {
                ButtonTone::Primary
            } else {
                ButtonTone::Ghost
            };
            if button(
                Rect::new(28.0 + index as f32 * 146.0, 154.0, 132.0, 32.0),
                filter,
                true,
                tone,
            ) {
                self.listing_filter = index;
            }
        }

        let visible: Vec<usize> = self
            .available_properties
            .iter()
            .enumerate()
            .filter(|(_, property)| listing_matches(self, property))
            .map(|(index, _)| index)
            .collect();

        let card_w = (ui_width() - 86.0) / 3.0;
        let card_h = 218.0;
        let mut inspect_index = None;

        for (slot, property_index) in visible.iter().enumerate() {
            let property = &self.available_properties[*property_index];
            let row = slot / 3;
            let col = slot % 3;
            let x = 28.0 + col as f32 * (card_w + 15.0);
            let y = 214.0 + row as f32 * (card_h + 16.0);
            let rect = Rect::new(x, y, card_w, card_h);
            soft_panel(rect);
            draw_house_art(Rect::new(x + 12.0, y + 12.0, card_w - 24.0, 82.0), property);

            label_fit(
                &property.address,
                x + 14.0,
                y + 122.0,
                card_w - 144.0,
                21,
                TEXT_BRIGHT,
            );
            label(
                reason_to_care(property, self),
                x + 14.0,
                y + 146.0,
                16,
                verdict_color(property, self),
            );
            label(
                &format!("Guide {}", format_money(property.guide_price)),
                x + 14.0,
                y + 174.0,
                19,
                POSITIVE,
            );
            draw_badge(
                upside_badge(property, self),
                Rect::new(x + 14.0, y + 188.0, 108.0, 24.0),
                POSITIVE,
            );
            draw_badge(
                risk_badge(property),
                Rect::new(x + 132.0, y + 188.0, 96.0, 24.0),
                risk_color(property),
            );
            draw_badge(
                demand_badge(property),
                Rect::new(x + 238.0, y + 188.0, 104.0, 24.0),
                crate::ui::BLUE,
            );

            let inspect_pressed = if button(
                Rect::new(x + card_w - 116.0, y + 142.0, 98.0, 34.0),
                "Inspect",
                true,
                ButtonTone::Primary,
            ) {
                true
            } else {
                rect_clicked(rect)
            };
            if inspect_pressed {
                inspect_index = Some(*property_index);
            }
        }

        if visible.is_empty() {
            let empty = Rect::new(28.0, 214.0, ui_width() - 56.0, 150.0);
            soft_panel(empty);
            label(
                "No listings match this filter.",
                empty.x + 20.0,
                empty.y + 48.0,
                24,
                TEXT_BRIGHT,
            );
            label(
                "Try All, or advance the week to refresh the market.",
                empty.x + 20.0,
                empty.y + 82.0,
                18,
                TEXT_DIM,
            );
        }

        if let Some(index) = inspect_index {
            self.open_property_detail(index);
        }
    }
}

fn listing_matches(app: &App, property: &Property) -> bool {
    match app.listing_filter {
        1 => property.hidden_defect_risk < 0.18,
        2 => upside_amount(property, app) >= 70_000,
        3 => property.buyer_demand >= 65,
        4 => property.guide_price <= 500_000,
        _ => true,
    }
}

fn upside_amount(property: &Property, app: &App) -> i64 {
    let (_, high) = estimated_value_range(property, app.market());
    high - property.guide_price
}

fn upside_badge(property: &Property, app: &App) -> &'static str {
    let upside = upside_amount(property, app);
    if upside >= 95_000 {
        "HIGH UPSIDE"
    } else if upside >= 55_000 {
        "GOOD UPSIDE"
    } else {
        "TIGHT DEAL"
    }
}

fn risk_badge(property: &Property) -> &'static str {
    if property.hidden_defect_risk >= 0.28 {
        "HIGH RISK"
    } else if property.hidden_defect_risk >= 0.16 {
        "CHECK RISK"
    } else {
        "LOW RISK"
    }
}

fn risk_color(property: &Property) -> Color {
    if property.hidden_defect_risk >= 0.28 {
        NEGATIVE
    } else if property.hidden_defect_risk >= 0.16 {
        WARNING
    } else {
        POSITIVE
    }
}

fn demand_badge(property: &Property) -> &'static str {
    if property.buyer_demand >= 70 {
        "HOT DEMAND"
    } else if property.buyer_demand >= 55 {
        "STEADY"
    } else {
        "SOFT ROOM"
    }
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
    } else if upside_amount(property, app) >= 95_000 || property.buyer_demand >= 70 {
        POSITIVE
    } else {
        TEXT_DIM
    }
}
