use crate::app::App;
use crate::sim::valuation::estimated_value_range;
use crate::ui::*;
use macroquad::prelude::*;

impl App {
    pub(crate) fn draw_property_list(&mut self) {
        label("Auction Listings", 28.0, 106.0, 30, TEXT_BRIGHT);
        label("Compare guide price, value range, condition, risk, and buyer demand before registering.", 30.0, 134.0, 18, TEXT_DIM);

        let card_w = (screen_width() - 86.0) / 3.0;
        let card_h = 244.0;
        let mut inspect_index = None;

        for (index, property) in self.available_properties.iter().enumerate() {
            let row = index / 3;
            let col = index % 3;
            let x = 28.0 + col as f32 * (card_w + 15.0);
            let y = 164.0 + row as f32 * (card_h + 16.0);
            let rect = Rect::new(x, y, card_w, card_h);
            panel(rect);
            draw_house_art(Rect::new(x + 12.0, y + 12.0, card_w - 24.0, 92.0), property);

            label(&property.address, x + 14.0, y + 132.0, 21, TEXT_BRIGHT);
            label(
                &format!(
                    "{} | {} bed, {} bath | {}sqm",
                    property.suburb, property.bedrooms, property.bathrooms, property.land_size
                ),
                x + 14.0,
                y + 156.0,
                16,
                TEXT_DIM,
            );
            let (low, high) = estimated_value_range(property, self.market());
            label(
                &format!(
                    "Guide {} | Est. {} - {}",
                    format_money(property.guide_price),
                    format_money(low),
                    format_money(high)
                ),
                x + 14.0,
                y + 182.0,
                17,
                TEXT,
            );
            draw_meter(
                "Demand",
                property.buyer_demand,
                Rect::new(x + 14.0, y + 206.0, 118.0, 10.0),
                POSITIVE,
            );
            draw_meter(
                "Appeal",
                property.appeal,
                Rect::new(x + 148.0, y + 206.0, 118.0, 10.0),
                ACCENT,
            );
            label(
                property.condition.label(),
                x + card_w - 104.0,
                y + 221.0,
                16,
                condition_color(property.condition),
            );

            if button(
                Rect::new(x + card_w - 116.0, y + card_h - 48.0, 98.0, 34.0),
                "Inspect",
                true,
                ButtonTone::Primary,
            ) {
                inspect_index = Some(index);
            }
        }

        if let Some(index) = inspect_index {
            self.open_property_detail(index);
        }
    }
}

fn condition_color(condition: crate::model::Condition) -> Color {
    match condition {
        crate::model::Condition::Rough => NEGATIVE,
        crate::model::Condition::Tired => WARNING,
        crate::model::Condition::Solid => POSITIVE,
        crate::model::Condition::Premium => crate::ui::BLUE,
    }
}
