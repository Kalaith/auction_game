use crate::app::App;
use crate::screens::Screen;
use crate::sim::valuation::net_worth;
use crate::ui::*;
use macroquad::prelude::*;

impl App {
    pub(crate) fn draw_dashboard(&mut self) {
        let margin = 28.0;
        let top = 92.0;
        let left = Rect::new(margin, top, 382.0, 248.0);
        panel(left);
        label(
            "Player Position",
            left.x + 18.0,
            left.y + 34.0,
            24,
            TEXT_BRIGHT,
        );
        draw_value(
            "Cash",
            &format_money(self.player.cash),
            left.x + 18.0,
            left.y + 78.0,
            left.w - 36.0,
        );
        draw_value(
            "Debt",
            &format_money(self.player.debt),
            left.x + 18.0,
            left.y + 116.0,
            left.w - 36.0,
        );
        draw_value(
            "Net worth",
            &format_money(net_worth(&self.player, self.market())),
            left.x + 18.0,
            left.y + 154.0,
            left.w - 36.0,
        );
        draw_value(
            "Owned properties",
            &self.player.properties.len().to_string(),
            left.x + 18.0,
            left.y + 192.0,
            left.w - 36.0,
        );

        let market = Rect::new(438.0, top, screen_width() - 466.0, 248.0);
        panel(market);
        label(
            &self.market().title,
            market.x + 18.0,
            market.y + 34.0,
            24,
            TEXT_BRIGHT,
        );
        let mut y = market.y + 72.0;
        let items = self.market().items.clone();
        for item in items {
            label("-", market.x + 22.0, y, 19, ACCENT);
            y = draw_wrapped_text(&item, market.x + 44.0, y, market.w - 70.0, 18, TEXT);
            y += 6.0;
        }

        let auctions = Rect::new(margin, 368.0, screen_width() - margin * 2.0, 250.0);
        panel(auctions);
        label(
            "Upcoming Auctions",
            auctions.x + 18.0,
            auctions.y + 34.0,
            24,
            TEXT_BRIGHT,
        );

        let card_w = (auctions.w - 72.0) / 3.0;
        let mut action = None;
        for (slot, property) in self.available_properties.iter().take(3).enumerate() {
            let x = auctions.x + 18.0 + slot as f32 * (card_w + 18.0);
            let rect = Rect::new(x, auctions.y + 58.0, card_w, 166.0);
            dark_panel(rect);
            draw_house_art(
                Rect::new(rect.x + 10.0, rect.y + 10.0, 138.0, 92.0),
                property,
            );
            label(
                &property.address,
                rect.x + 160.0,
                rect.y + 32.0,
                20,
                TEXT_BRIGHT,
            );
            label(
                &property.suburb,
                rect.x + 160.0,
                rect.y + 58.0,
                17,
                TEXT_DIM,
            );
            label(
                &format!("Guide {}", format_money(property.guide_price)),
                rect.x + 160.0,
                rect.y + 86.0,
                18,
                TEXT,
            );
            if button(
                Rect::new(rect.x + 160.0, rect.y + 112.0, 118.0, 36.0),
                "Inspect",
                true,
                ButtonTone::Primary,
            ) {
                action = Some(slot);
            }
        }

        if button(
            Rect::new(screen_width() - 196.0, 632.0, 168.0, 42.0),
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
}
