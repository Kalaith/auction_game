use crate::app::App;
use crate::model::PropertyId;
use crate::screens::portfolio_widgets::{
    draw_active_project_decision, draw_contractor_selector, draw_empty_portfolio,
    draw_hold_decision, draw_marketing_selector, draw_problem_card, draw_sell_decision,
    draw_upgrade_decision, recommended_upgrade,
};
use crate::sim::finance::borrowing_limit;
use crate::sim::valuation::current_value;
use crate::ui::*;
use macroquad::prelude::*;

impl App {
    pub(crate) fn draw_portfolio(&mut self) {
        label("Next Move", 28.0, 106.0, 30, TEXT_BRIGHT);
        if self.player.properties.is_empty() {
            draw_empty_portfolio(self);
            return;
        }

        let owned = self.player.properties[0].clone();
        let estimate = current_value(&owned, self.market());
        let position = estimate
            - owned.purchase_price
            - owned.purchase_fees
            - owned.upgrade_spend()
            - owned.holding_spend();
        let bank_room = borrowing_limit(&self.player, self.market()) - self.player.debt;
        let has_active_project = owned.active_renovation.is_some();
        let main = Rect::new(28.0, 142.0, ui_width() - 56.0, ui_height() - 184.0);
        soft_panel(main);

        draw_house_art(
            Rect::new(main.x + 16.0, main.y + 16.0, 330.0, 208.0),
            &owned.property,
        );
        label(
            &owned.property.address,
            main.x + 370.0,
            main.y + 42.0,
            30,
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
            main.y + 72.0,
            17,
            TEXT_DIM,
        );
        label(
            &format!("Deposit paid {}", format_money(owned.deposit_paid)),
            main.x + 372.0,
            main.y + 92.0,
            15,
            TEXT_DIM,
        );

        let stat_y = main.y + 112.0;
        draw_money_stat(
            "Current Estimate",
            &format_money(estimate),
            "Market adjusted",
            Rect::new(main.x + 370.0, stat_y, 220.0, 78.0),
            if position >= 0 { POSITIVE } else { WARNING },
        );
        draw_money_stat(
            "Projected Position",
            &format_money(position),
            "Before sale result",
            Rect::new(main.x + 606.0, stat_y, 220.0, 78.0),
            if position >= 0 { POSITIVE } else { NEGATIVE },
        );
        draw_problem_card(
            Rect::new(main.x + 842.0, stat_y, main.w - 862.0, 128.0),
            &owned,
            bank_room,
            self.market(),
        );

        label(
            "Recommended Actions",
            main.x + 18.0,
            main.y + 264.0,
            24,
            TEXT_BRIGHT,
        );
        let card_y = main.y + 286.0;
        let card_w = (main.w - 54.0) / 3.0;
        let mut upgrade_action: Option<(PropertyId, String)> = None;
        let mut hold_week = false;

        if has_active_project {
            draw_active_project_decision(Rect::new(main.x + 18.0, card_y, card_w, 160.0), &owned);
        } else if let Some((upgrade, quote)) = recommended_upgrade(self, &owned) {
            if draw_upgrade_decision(
                Rect::new(main.x + 18.0, card_y, card_w, 160.0),
                upgrade,
                &quote,
                self.player.cash,
                owned.has_upgrade(&upgrade.id),
            ) {
                upgrade_action = Some((owned.property.id, upgrade.id.clone()));
            }
        }

        if draw_hold_decision(
            Rect::new(main.x + 36.0 + card_w, card_y, card_w, 160.0),
            &owned,
            self.campaign_status.is_finished(),
        ) {
            hold_week = true;
        }

        let sale_action = draw_sell_decision(
            Rect::new(main.x + 54.0 + card_w * 2.0, card_y, card_w, 160.0),
            position,
            has_active_project,
            self.selected_marketing_plan,
            self.player.cash,
        );

        draw_contractor_selector(self, main, has_active_project);
        draw_marketing_selector(self, main, &owned);

        if let Some((property_id, upgrade_id)) = upgrade_action {
            self.buy_upgrade(property_id, &upgrade_id);
        }
        if let Some(choice) = sale_action {
            self.sell_property(owned.property.id, choice);
        } else if hold_week {
            self.advance_week();
        }
    }
}
