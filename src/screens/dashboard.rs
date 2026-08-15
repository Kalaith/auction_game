use crate::app::App;
use crate::model::{
    CampaignStatus, Property, CAMPAIGN_GOAL_NET_WORTH, CAMPAIGN_GOAL_PROPERTIES,
    CAMPAIGN_GOAL_WEEKLY_RENT,
};
use crate::screens::Screen;
use crate::sim::campaign::{
    assess_campaign, campaign_progress, next_unlock_note, portfolio_weekly_cashflow,
};
use crate::sim::finance::max_financeable_bid;
use crate::sim::valuation::{estimated_value_range, net_worth};
use crate::ui::*;
use macroquad::prelude::*;

impl App {
    pub(crate) fn draw_dashboard(&mut self) {
        let margin = 28.0;
        let top = 92.0;
        let width = ui_width() - margin * 2.0;
        let current_net_worth = net_worth(&self.player, self.market());
        if self.campaign_status.is_finished() {
            self.draw_campaign_conclusion(current_net_worth);
            return;
        }

        self.draw_dashboard_stats(margin, top, width, current_net_worth);
        let pulse_gap = 14.0;
        let pulse_width = width * 0.59;
        self.draw_market_pulse(margin, top + 98.0, pulse_width, current_net_worth);
        self.draw_weekly_statement(
            margin + pulse_width + pulse_gap,
            top + 98.0,
            width - pulse_width - pulse_gap,
        );

        label(
            "Featured Opportunities",
            margin,
            top + 288.0,
            25,
            TEXT_BRIGHT,
        );
        label(
            &format!(
                "Season: {} attended  |  {} bought  |  {} disciplined exits  |  {} rent reviews",
                self.player.career.auctions_attended,
                self.player.career.homes_bought,
                self.player.career.disciplined_walkaways,
                self.player.career.rent_reviews_completed
            ),
            margin + 330.0,
            top + 288.0,
            16,
            TEXT_DIM,
        );

        let card_w = (width - 36.0) / 3.0;
        let mut action = None;
        for (slot, property) in self.available_properties.iter().take(3).enumerate() {
            let x = margin + slot as f32 * (card_w + 18.0);
            let rect = Rect::new(x, top + 314.0, card_w, 164.0);
            if slot == 0 {
                highlight_panel(rect);
            } else {
                soft_panel(rect);
            }
            if dashboard_property_card(self, rect, property) {
                action = Some(slot);
            }
        }

        let action_y = ui_height() - 88.0;
        if button(
            Rect::new(ui_width() - 382.0, action_y, 168.0, 40.0),
            "Advance Week",
            !self.campaign_status.is_finished(),
            ButtonTone::Ghost,
        ) {
            self.advance_week();
        }
        if button(
            Rect::new(ui_width() - 196.0, action_y, 168.0, 40.0),
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

    fn draw_campaign_conclusion(&mut self, net_worth_value: i64) {
        let won = self.campaign_status == CampaignStatus::Won;
        let rental = crate::sim::rental::portfolio_rental_snapshot(&self.player);
        let weekly_cashflow = portfolio_weekly_cashflow(&self.player, self.market());
        let assessment = assess_campaign(&self.player, self.market());
        let title = if won {
            "Portfolio Established"
        } else {
            "The Bank Closes The Campaign"
        };
        let color = if won { POSITIVE } else { WARNING };
        label(title, 80.0, 132.0, 38, TEXT_BRIGHT);
        draw_wrapped_text(
            if won {
                "Three doors, dependable rent, and enough equity to keep growing: the first portfolio now works without pretending every auction was a win."
            } else {
                "Week 24 has passed. The ledger now shows exactly what constrained this portfolio and what to change next season."
            },
            82.0,
            166.0,
            ui_width() - 164.0,
            19,
            TEXT_DIM,
        );

        let panel = Rect::new(80.0, 218.0, ui_width() - 160.0, 260.0);
        highlight_panel(panel);
        label("FINAL LEDGER", panel.x + 22.0, panel.y + 34.0, 17, color);
        let gap = 16.0;
        let card_w = (panel.w - 76.0) / 3.0;
        let stats = [
            (
                "Homes",
                format!(
                    "{} / {}",
                    self.player.properties.len(),
                    CAMPAIGN_GOAL_PROPERTIES
                ),
                if won || assessment.homes_short == 0 {
                    "Goal secured".to_string()
                } else {
                    format!("Short {}", plural_homes(assessment.homes_short))
                },
            ),
            (
                "Weekly Rent",
                format_money(rental.gross_rent),
                if won || assessment.rent_short == 0 {
                    format!("Goal secured | Net {} / wk", signed_money(weekly_cashflow))
                } else {
                    format!("Short {} / week", format_money(assessment.rent_short))
                },
            ),
            (
                "Net Worth",
                format_money(net_worth_value),
                if won || assessment.net_worth_short == 0 {
                    "Goal secured".to_string()
                } else {
                    format!("Short {}", format_money(assessment.net_worth_short))
                },
            ),
        ];
        for (index, (name, value, note)) in stats.iter().enumerate() {
            draw_money_stat(
                name,
                value,
                note,
                Rect::new(
                    panel.x + 22.0 + index as f32 * (card_w + gap),
                    panel.y + 64.0,
                    card_w,
                    96.0,
                ),
                color,
            );
        }
        label(
            &format!(
                "Cash {}  |  Debt {}  |  Reputation {:+}",
                format_money(self.player.cash),
                format_money(self.player.debt),
                self.player.reputation
            ),
            panel.x + 24.0,
            panel.y + 204.0,
            18,
            TEXT,
        );
        label(
            &format!(
                "Auction room: {} attended  |  {} bought ({} after pass-in)  |  {} disciplined exits  |  {} registrations passed",
                self.player.career.auctions_attended,
                self.player.career.homes_bought,
                self.player.career.post_auction_buys,
                self.player.career.disciplined_walkaways,
                self.player.career.unused_registrations,
            ),
            panel.x + 24.0,
            panel.y + 232.0,
            15,
            TEXT_DIM,
        );
        label(
            &format!(
                "Portfolio: {} sold  |  realized {}  |  {} rent reviews ({} vacancies)  |  final rent {} / week",
                self.player.career.homes_sold,
                signed_money(self.player.career.realized_profit),
                self.player.career.rent_reviews_completed,
                self.player.career.review_vacancies,
                format_money(rental.gross_rent)
            ),
            panel.x + 24.0,
            panel.y + 252.0,
            15,
            TEXT_DIM,
        );

        let advice = Rect::new(80.0, 492.0, ui_width() - 160.0, 72.0);
        let advice_copy = if won && weekly_cashflow < 0 {
            format!(
                "The brief is secured, but operations still lose {} each week. Pay down the worst loan, lift supported rent, or recycle a weak asset.",
                format_money(weekly_cashflow.abs())
            )
        } else {
            assessment.priority_advice().to_string()
        };
        soft_panel(advice);
        label(
            if won && weekly_cashflow < 0 {
                "NEXT CYCLE"
            } else if won {
                "WHAT WORKED"
            } else {
                "BINDING CONSTRAINT"
            },
            advice.x + 18.0,
            advice.y + 25.0,
            15,
            color,
        );
        draw_wrapped_text(
            &advice_copy,
            advice.x + 190.0,
            advice.y + 22.0,
            advice.w - 208.0,
            16,
            TEXT,
        );

        if button(
            Rect::new(80.0, 584.0, 230.0, 48.0),
            "REVIEW PORTFOLIO",
            !self.player.properties.is_empty(),
            ButtonTone::Secondary,
        ) {
            self.screen = Screen::Portfolio;
        }
        if button(
            Rect::new(ui_width() - 330.0, 584.0, 250.0, 48.0),
            "START NEW PORTFOLIO",
            true,
            ButtonTone::Primary,
        ) {
            self.start_new_game();
        }
    }

    fn draw_dashboard_stats(&self, x: f32, y: f32, width: f32, net_worth_value: i64) {
        let gap = 12.0;
        let card_w = (width - gap * 3.0) / 4.0;
        let (property_count, weekly_rent, _) = campaign_progress(&self.player, self.market());
        let buying_power = max_financeable_bid(&self.player, self.market());
        let reputation_note = format!("Rep {:+}", self.player.reputation);
        let stats = [
            (
                "Cash",
                format_compact_money(self.player.cash),
                "Ready capital".to_string(),
                POSITIVE,
            ),
            (
                "Buying Power",
                format_compact_money(buying_power),
                "Max next bid".to_string(),
                POSITIVE,
            ),
            (
                "Portfolio",
                format!("{property_count}/{CAMPAIGN_GOAL_PROPERTIES}"),
                format!(
                    "Net worth {} / {}",
                    format_compact_money(net_worth_value),
                    format_compact_money(CAMPAIGN_GOAL_NET_WORTH)
                ),
                ACCENT,
            ),
            (
                "Weekly Rent",
                format_compact_money(weekly_rent),
                format!(
                    "Goal {} | {reputation_note}",
                    format_money(CAMPAIGN_GOAL_WEEKLY_RENT)
                ),
                crate::ui::BLUE,
            ),
        ];

        for (index, (title, value, note, color)) in stats.iter().enumerate() {
            let rect = Rect::new(x + index as f32 * (card_w + gap), y, card_w, 78.0);
            draw_money_stat(title, value, note, rect, *color);
        }
    }

    fn draw_market_pulse(&self, x: f32, y: f32, width: f32, net_worth_value: i64) {
        let rect = Rect::new(x, y, width, 158.0);
        soft_panel(rect);
        label(
            "Market Pulse",
            rect.x + 18.0,
            rect.y + 32.0,
            25,
            TEXT_BRIGHT,
        );
        label(
            &self.market().title,
            rect.x + 210.0,
            rect.y + 32.0,
            15,
            TEXT_DIM,
        );
        draw_badge(
            "THIS WEEK",
            Rect::new(rect.x + rect.w - 118.0, rect.y + 17.0, 92.0, 26.0),
            crate::ui::BLUE,
        );

        for (index, item) in self.market().items.iter().take(2).enumerate() {
            let y = rect.y + 62.0 + index as f32 * 27.0;
            let (headline, _) = split_market_line(item);
            label_fit(&headline, rect.x + 28.0, y, rect.w - 190.0, 18, TEXT_BRIGHT);
        }
        label_fit(
            &self.market().strategy_effect,
            rect.x + 28.0,
            rect.y + 112.0,
            rect.w - 190.0,
            16,
            ACCENT,
        );

        let unlock = Rect::new(rect.x + 18.0, rect.y + rect.h - 30.0, rect.w - 36.0, 24.0);
        draw_rectangle(
            unlock.x,
            unlock.y,
            unlock.w,
            unlock.h,
            Color::new(ACCENT.r * 0.20, ACCENT.g * 0.16, ACCENT.b * 0.08, 1.0),
        );
        label("NEXT", unlock.x + 12.0, unlock.y + 17.0, 14, ACCENT);
        label_fit(
            next_unlock_note(self.week, net_worth_value, self.player.reputation),
            unlock.x + 70.0,
            unlock.y + 17.0,
            unlock.w - 82.0,
            14,
            TEXT_DIM,
        );
    }

    fn draw_weekly_statement(&self, x: f32, y: f32, width: f32) {
        let rect = Rect::new(x, y, width, 158.0);
        soft_panel(rect);
        label(
            "Weekly Statement",
            rect.x + 18.0,
            rect.y + 30.0,
            23,
            TEXT_BRIGHT,
        );
        let Some(pressure) = &self.last_weekly_pressure else {
            draw_badge(
                "PENDING",
                Rect::new(rect.x + rect.w - 104.0, rect.y + 15.0, 82.0, 25.0),
                TEXT_DIM,
            );
            draw_wrapped_text(
                "Advance the week to collect rent and close the first portfolio statement.",
                rect.x + 18.0,
                rect.y + 66.0,
                rect.w - 36.0,
                17,
                TEXT_DIM,
            );
            return;
        };

        let net = pressure.rental_income - pressure.total;
        draw_badge(
            if net >= 0 { "CASHFLOW +" } else { "CASHFLOW -" },
            Rect::new(rect.x + rect.w - 122.0, rect.y + 15.0, 100.0, 25.0),
            if net >= 0 { POSITIVE } else { WARNING },
        );
        let rows = [
            ("Rent collected", pressure.rental_income),
            ("Management", -pressure.rental_operating_cost),
            ("Loan interest", -pressure.debt_interest),
            ("Property costs", -pressure.holding_cost),
            ("Net movement", net),
        ];
        for (index, (title, amount)) in rows.iter().enumerate() {
            let row_y = rect.y + 56.0 + index as f32 * 20.0;
            label(title, rect.x + 18.0, row_y, 14, TEXT_DIM);
            label(
                &signed_money(*amount),
                rect.x + rect.w - 112.0,
                row_y,
                14,
                if *amount >= 0 { POSITIVE } else { WARNING },
            );
        }
        if pressure.shortfall_added_to_debt > 0 {
            label_fit(
                &format!(
                    "Shortfall {} added to debt",
                    format_money(pressure.shortfall_added_to_debt)
                ),
                rect.x + 18.0,
                rect.y + rect.h - 8.0,
                rect.w - 36.0,
                13,
                NEGATIVE,
            );
        }
    }
}

fn plural_homes(count: usize) -> String {
    format!("{} {}", count, if count == 1 { "home" } else { "homes" })
}

fn signed_money(amount: i64) -> String {
    if amount >= 0 {
        format!("+{}", format_money(amount))
    } else {
        format!("-{}", format_money(amount.abs()))
    }
}

fn dashboard_property_card(app: &App, rect: Rect, property: &Property) -> bool {
    draw_house_art(
        Rect::new(rect.x + 14.0, rect.y + 18.0, 146.0, 94.0),
        property,
    );
    let text_x = rect.x + 178.0;
    let text_w = rect.w - 198.0;
    label_fit(
        &property.address,
        text_x,
        rect.y + 38.0,
        text_w,
        21,
        TEXT_BRIGHT,
    );
    label(&property.suburb, text_x, rect.y + 66.0, 16, TEXT_DIM);
    label(
        reason_to_care(property, app),
        text_x,
        rect.y + 90.0,
        16,
        verdict_color(property, app),
    );
    label(
        &format!("Guide {}", format_money(property.guide_price)),
        text_x,
        rect.y + 116.0,
        20,
        POSITIVE,
    );
    let pressed = button(
        Rect::new(text_x, rect.y + 130.0, text_w, 30.0),
        "View Details",
        true,
        ButtonTone::Primary,
    );
    pressed || rect_clicked(rect)
}

fn split_market_line(item: &str) -> (String, String) {
    let Some((headline, rest)) = item.split_once('.') else {
        return (item.to_string(), String::new());
    };
    (headline.to_string(), rest.trim().to_string())
}

fn reason_to_care(property: &Property, app: &App) -> &'static str {
    let _ = app;
    property.deal_archetype.label()
}

fn verdict_color(property: &Property, app: &App) -> Color {
    if property.hidden_defect_risk >= 0.28 {
        WARNING
    } else if upside_amount(property, app) >= 95_000 {
        POSITIVE
    } else {
        TEXT_DIM
    }
}

fn upside_amount(property: &Property, app: &App) -> i64 {
    let (_, high) = estimated_value_range(property, app.market());
    high - property.guide_price
}
