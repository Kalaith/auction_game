use crate::app::App;
use crate::model::{ContractorTier, PropertyId};
use crate::screens::Screen;
use crate::sim::renovation::quote_renovation;
use crate::sim::sale_sim::ReserveChoice;
use crate::sim::valuation::current_value;
use crate::ui::*;
use macroquad::prelude::*;

impl App {
    pub(crate) fn draw_portfolio(&mut self) {
        label("Owned Property", 28.0, 106.0, 30, TEXT_BRIGHT);
        if self.player.properties.is_empty() {
            let rect = Rect::new(28.0, 142.0, screen_width() - 56.0, 220.0);
            panel(rect);
            label(
                "No properties owned",
                rect.x + 20.0,
                rect.y + 42.0,
                26,
                TEXT_BRIGHT,
            );
            draw_wrapped_text(
                "Buy a property at auction, then decide whether the renovation spend is worth the resale risk.",
                rect.x + 20.0,
                rect.y + 84.0,
                rect.w - 40.0,
                19,
                TEXT,
            );
            if button(
                Rect::new(rect.x + 20.0, rect.y + 146.0, 164.0, 42.0),
                "Find Auctions",
                true,
                ButtonTone::Primary,
            ) {
                self.screen = Screen::PropertyList;
            }
            return;
        }

        let owned = self.player.properties[0].clone();
        let main = Rect::new(28.0, 142.0, screen_width() - 56.0, 520.0);
        panel(main);
        draw_house_art(
            Rect::new(main.x + 16.0, main.y + 16.0, 330.0, 210.0),
            &owned.property,
        );
        label(
            &owned.property.address,
            main.x + 370.0,
            main.y + 42.0,
            29,
            TEXT_BRIGHT,
        );
        label(
            &format!(
                "Bought for {} | Held {} weeks | {}",
                format_money(owned.purchase_price),
                owned.weeks_held,
                owned.property.condition.label()
            ),
            main.x + 372.0,
            main.y + 74.0,
            18,
            TEXT_DIM,
        );

        let estimate = current_value(&owned, self.market());
        let stats = Rect::new(main.x + 370.0, main.y + 104.0, 390.0, 150.0);
        dark_panel(stats);
        draw_value(
            "Current estimate",
            &format_money(estimate),
            stats.x + 16.0,
            stats.y + 36.0,
            stats.w - 32.0,
        );
        draw_value(
            "Deposit paid",
            &format_money(owned.deposit_paid),
            stats.x + 16.0,
            stats.y + 70.0,
            stats.w - 32.0,
        );
        draw_value(
            "Upgrade spend",
            &format_money(owned.upgrade_spend()),
            stats.x + 16.0,
            stats.y + 104.0,
            stats.w - 32.0,
        );
        draw_value(
            "Holding costs",
            &format_money(owned.holding_spend()),
            stats.x + 16.0,
            stats.y + 138.0,
            stats.w - 32.0,
        );

        let risk = Rect::new(main.x + 786.0, main.y + 104.0, main.w - 806.0, 150.0);
        dark_panel(risk);
        label("Risk Read", risk.x + 16.0, risk.y + 32.0, 22, TEXT_BRIGHT);
        let risk_text = if owned.hidden_defect_discovered && !owned.has_defect_repair() {
            "Inspection has found a structural concern. Buyers will punish it unless repaired."
        } else if owned.hidden_defect_discovered {
            "The major defect has been handled. Buyers should focus on presentation again."
        } else {
            "No major hidden defect has surfaced yet. That does not make the house risk-free."
        };
        draw_wrapped_text(
            risk_text,
            risk.x + 16.0,
            risk.y + 64.0,
            risk.w - 32.0,
            17,
            TEXT,
        );
        if let Some(last_upgrade) = owned.upgrades.last() {
            draw_wrapped_text(
                &format!(
                    "Last job: {} contractor, {} weeks. {}",
                    last_upgrade.contractor.label(),
                    last_upgrade.weeks_taken,
                    last_upgrade.note
                ),
                risk.x + 16.0,
                risk.y + 112.0,
                risk.w - 32.0,
                14,
                TEXT_DIM,
            );
        }

        label(
            "Renovation Options",
            main.x + 18.0,
            main.y + 268.0,
            24,
            TEXT_BRIGHT,
        );
        draw_contractor_selector(self, main);
        let mut upgrade_action: Option<(PropertyId, String)> = None;
        for (index, upgrade) in self.data.upgrades.iter().enumerate() {
            let quote = quote_renovation(&owned, upgrade, self.selected_contractor, self.market());
            let row = index / 3;
            let col = index % 3;
            let x = main.x + 18.0 + col as f32 * 390.0;
            let y = main.y + 292.0 + row as f32 * 72.0;
            let rect = Rect::new(x, y, 370.0, 68.0);
            dark_panel(rect);
            label(&upgrade.name, x + 12.0, y + 24.0, 19, TEXT_BRIGHT);
            label(
                &format!(
                    "{} | {}w | +{} value",
                    format_money(quote.cash_outlay),
                    quote.holding_weeks,
                    format_money(quote.value_boost)
                ),
                x + 12.0,
                y + 46.0,
                15,
                TEXT_DIM,
            );
            label(
                if quote.is_overcapitalized {
                    "Overcapitalising"
                } else if quote.permit_risk > 0 {
                    "Permit risk"
                } else {
                    &quote.note
                },
                x + 12.0,
                y + 62.0,
                13,
                if quote.is_overcapitalized {
                    WARNING
                } else {
                    TEXT_DIM
                },
            );
            let already_done = owned.has_upgrade(&upgrade.id);
            let enabled = !already_done && self.player.cash >= quote.cash_outlay;
            let label_text = if already_done { "Done" } else { "Buy" };
            if button(
                Rect::new(x + 290.0, y + 15.0, 62.0, 34.0),
                label_text,
                enabled,
                ButtonTone::Secondary,
            ) {
                upgrade_action = Some((owned.property.id, upgrade.id.clone()));
            }
        }

        label(
            "Sell At Auction",
            main.x + 18.0,
            main.y + 456.0,
            24,
            TEXT_BRIGHT,
        );
        let mut sale_action = None;
        if button(
            Rect::new(main.x + 220.0, main.y + 470.0, 170.0, 42.0),
            "Conservative",
            true,
            ButtonTone::Primary,
        ) {
            sale_action = Some(ReserveChoice::Conservative);
        }
        if button(
            Rect::new(main.x + 408.0, main.y + 470.0, 130.0, 42.0),
            "Fair",
            true,
            ButtonTone::Secondary,
        ) {
            sale_action = Some(ReserveChoice::Fair);
        }
        if button(
            Rect::new(main.x + 556.0, main.y + 470.0, 150.0, 42.0),
            "Ambitious",
            true,
            ButtonTone::Danger,
        ) {
            sale_action = Some(ReserveChoice::Ambitious);
        }

        if let Some((property_id, upgrade_id)) = upgrade_action {
            self.buy_upgrade(property_id, &upgrade_id);
        }
        if let Some(choice) = sale_action {
            self.sell_property(owned.property.id, choice);
        }
    }
}

fn draw_contractor_selector(app: &mut App, main: Rect) {
    label(
        "Contractor",
        main.x + 824.0,
        main.y + 268.0,
        18,
        TEXT_BRIGHT,
    );

    let options = [
        ContractorTier::Budget,
        ContractorTier::Reliable,
        ContractorTier::Premium,
    ];
    for (index, tier) in options.iter().enumerate() {
        let selected = app.selected_contractor == *tier;
        let tone = if selected {
            ButtonTone::Primary
        } else {
            ButtonTone::Ghost
        };
        if button(
            Rect::new(
                main.x + 926.0 + index as f32 * 78.0,
                main.y + 246.0,
                72.0,
                30.0,
            ),
            tier.label(),
            true,
            tone,
        ) {
            app.selected_contractor = *tier;
            app.status = format!(
                "{} contractor selected for future renovations.",
                tier.label()
            );
        }
    }
}
