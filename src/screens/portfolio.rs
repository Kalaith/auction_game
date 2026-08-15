use crate::app::App;
use crate::model::PropertyId;
use crate::screens::portfolio_finance_widgets::{draw_loan_control, LoanAction};
use crate::screens::portfolio_rent_review::{draw_rent_review_decision, RentReviewChoice};
use crate::screens::portfolio_widgets::{
    draw_active_project_decision, draw_contractor_selector, draw_empty_portfolio,
    draw_hold_decision, draw_lease_decision, draw_maintenance_decision, draw_marketing_selector,
    draw_problem_card, draw_rental_campaign, draw_sell_decision, draw_skip_renovation,
    draw_upgrade_decision, recommended_upgrade,
};
use crate::sim::campaign::{
    annual_interest_rate_percent, weekly_debt_interest, weekly_holding_cost,
};
use crate::sim::finance::{borrowing_limit, property_cashflow, refinance_capacity};
use crate::sim::rental::{
    leasing_cost, portfolio_rental_snapshot, proposed_review_rent, rent_review_due,
    weekly_rent_for_owned,
};
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

        self.portfolio_index = self
            .portfolio_index
            .min(self.player.properties.len().saturating_sub(1));
        let rental = portfolio_rental_snapshot(&self.player);
        let portfolio_has_due_review = self.player.properties.iter().any(rent_review_due);
        let portfolio_cashflow = rental.gross_rent
            - rental.operating_cost
            - weekly_debt_interest(self.player.debt, self.market())
            - weekly_holding_cost(&self.player);
        label(
            &format!(
                "{} homes | {} leased | {} gross rent | {} weekly cashflow",
                self.player.properties.len(),
                self.player
                    .properties
                    .iter()
                    .filter(|owned| owned.is_leased)
                    .count(),
                format_money(rental.gross_rent),
                format_money(portfolio_cashflow)
            ),
            212.0,
            106.0,
            17,
            if portfolio_cashflow >= 0 {
                POSITIVE
            } else {
                WARNING
            },
        );

        let count = self.player.properties.len().min(6);
        let selector_gap = 10.0;
        let selector_w =
            (ui_width() - 56.0 - selector_gap * (count.saturating_sub(1)) as f32) / count as f32;
        let mut selected = None;
        for (index, property) in self.player.properties.iter().take(6).enumerate() {
            let property_week = property_cashflow(property, self.market());
            let rect = Rect::new(
                28.0 + index as f32 * (selector_w + selector_gap),
                124.0,
                selector_w,
                68.0,
            );
            if index == self.portfolio_index {
                highlight_panel(rect);
            } else {
                soft_panel(rect);
            }
            label_fit(
                &property.property.address,
                rect.x + 10.0,
                rect.y + 25.0,
                rect.w - 20.0,
                16,
                TEXT_BRIGHT,
            );
            let lease_label = if property.leasing_weeks_remaining > 0 {
                format!(
                    "On rental market | {} / wk",
                    format_money(property.weekly_rent)
                )
            } else if rent_review_due(property) {
                format!(
                    "Rent review due | {} / wk",
                    format_money(property.weekly_rent)
                )
            } else if property.is_leased {
                if let Some(issue) = &property.maintenance_issue {
                    format!(
                        "{} | {} net / wk",
                        issue.kind.label(),
                        format_money(property_week.net_cashflow)
                    )
                } else {
                    format!(
                        "Leased {} / wk | {} net",
                        format_money(property.weekly_rent),
                        format_money(property_week.net_cashflow)
                    )
                }
            } else {
                "Vacant".to_string()
            };
            label(
                &lease_label,
                rect.x + 10.0,
                rect.y + 50.0,
                14,
                if property.maintenance_issue.is_some() {
                    NEGATIVE
                } else if rent_review_due(property) {
                    WARNING
                } else if property.is_leased && property_week.net_cashflow >= 0 {
                    POSITIVE
                } else if property.is_leased {
                    WARNING
                } else if property.leasing_weeks_remaining > 0 {
                    crate::ui::BLUE
                } else {
                    WARNING
                },
            );
            if rect_clicked(rect) {
                selected = Some(index);
            }
        }
        if let Some(index) = selected {
            self.portfolio_index = index;
        }

        let owned = self.player.properties[self.portfolio_index].clone();
        let estimate = current_value(&owned, self.market());
        let position = estimate
            - owned.purchase_price
            - owned.purchase_fees
            - owned.upgrade_spend()
            - owned.holding_spend();
        let bank_room = borrowing_limit(&self.player, self.market()) - self.player.debt;
        let has_active_project = owned.active_renovation.is_some();
        let main = Rect::new(28.0, 208.0, ui_width() - 56.0, ui_height() - 250.0);
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
        label_fit(
            &format!(
                "Bought for {} | Held {} weeks | {} | Loan {:.1}% p.a.",
                format_money(owned.purchase_price),
                owned.weeks_held,
                owned.property.condition.label(),
                annual_interest_rate_percent(self.market())
            ),
            main.x + 372.0,
            main.y + 72.0,
            main.w - 690.0,
            17,
            TEXT_DIM,
        );
        label(
            &format!(
                "Deposit {} | Rent earned {} | Rental profit {}",
                format_money(owned.deposit_paid),
                format_money(owned.rent_received),
                format_money(owned.rental_profit())
            ),
            main.x + 372.0,
            main.y + 92.0,
            15,
            TEXT_DIM,
        );

        let stat_y = main.y + 112.0;
        draw_money_stat(
            "Current Estimate",
            &format_money(estimate),
            &format!("Equity {}", format_money(estimate - owned.debt)),
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
        let loan_action = draw_loan_control(
            Rect::new(main.x + main.w - 298.0, main.y + 14.0, 280.0, 82.0),
            &owned,
            self.player.cash,
            refinance_capacity(&self.player, owned.property.id, self.market()),
            if estimate > 0 {
                owned.debt as f32 / estimate as f32 * 100.0
            } else {
                0.0
            },
        );

        let card_y = main.y + 272.0;
        let card_w = (main.w - 54.0) / 3.0;
        let mut upgrade_action: Option<(PropertyId, String)> = None;
        let mut hold_week = false;
        let mut lease = false;
        let mut end_tenancy_action = false;
        let mut repair_maintenance_action = false;
        let mut rent_review_action = None;

        if owned.maintenance_issue.is_some() {
            if draw_maintenance_decision(
                Rect::new(main.x + 18.0, card_y, card_w, 160.0),
                &owned,
                self.player.cash,
            ) {
                repair_maintenance_action = true;
            }
        } else if rent_review_due(&owned) {
            rent_review_action = draw_rent_review_decision(
                Rect::new(main.x + 18.0, card_y, card_w, 160.0),
                &owned,
                proposed_review_rent(&owned, self.market()),
            );
        } else if has_active_project {
            draw_active_project_decision(Rect::new(main.x + 18.0, card_y, card_w, 160.0), &owned);
        } else if owned.leasing_weeks_remaining > 0 {
            draw_rental_campaign(Rect::new(main.x + 18.0, card_y, card_w, 160.0), &owned);
        } else if owned.is_leased {
            if draw_skip_renovation(
                Rect::new(main.x + 18.0, card_y, card_w, 160.0),
                &owned,
                self.player.cash,
            ) {
                end_tenancy_action = true;
            }
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
        } else {
            draw_skip_renovation(
                Rect::new(main.x + 18.0, card_y, card_w, 160.0),
                &owned,
                self.player.cash,
            );
        }

        let hold_rect = Rect::new(main.x + 36.0 + card_w, card_y, card_w, 160.0);
        if owned.is_leased || owned.leasing_weeks_remaining > 0 {
            if draw_hold_decision(
                hold_rect,
                &owned,
                property_cashflow(&owned, self.market()).net_cashflow,
                portfolio_has_due_review,
                self.campaign_status.is_finished(),
            ) {
                hold_week = true;
            }
        } else {
            let asking_rent = weekly_rent_for_owned(&owned, self.market());
            if draw_lease_decision(
                hold_rect,
                asking_rent,
                leasing_cost(asking_rent),
                self.player.cash,
                has_active_project
                    || (owned.hidden_defect_discovered && !owned.has_defect_repair()),
            ) {
                lease = true;
            }
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

        if loan_action == Some(LoanAction::PayDown) {
            self.pay_down_property_debt(owned.property.id);
        } else if loan_action == Some(LoanAction::Refinance) {
            self.refinance_owned_property(owned.property.id);
        } else if let Some((property_id, upgrade_id)) = upgrade_action {
            self.buy_upgrade(property_id, &upgrade_id);
        }
        if repair_maintenance_action {
            self.repair_property_maintenance(owned.property.id);
        } else if let Some(choice) = rent_review_action {
            self.review_property_rent(owned.property.id, choice == RentReviewChoice::TestMarket);
        } else if end_tenancy_action {
            self.end_property_tenancy(owned.property.id);
        } else if lease {
            self.lease_property(owned.property.id);
        } else if let Some(choice) = sale_action {
            self.sell_property(owned.property.id, choice);
        } else if hold_week {
            self.advance_week();
        }
    }
}
