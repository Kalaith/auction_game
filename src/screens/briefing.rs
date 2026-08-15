use crate::app::App;
use crate::model::{
    CAMPAIGN_GOAL_NET_WORTH, CAMPAIGN_GOAL_PROPERTIES, CAMPAIGN_GOAL_WEEKLY_RENT,
    CAMPAIGN_MAX_WEEKS,
};
use crate::screens::Screen;
use crate::ui::*;
use macroquad::prelude::*;

impl App {
    pub(crate) fn draw_briefing(&mut self) {
        let width = ui_width() - 160.0;
        let x = 80.0;
        label("Your First Saturday", x, 118.0, 34, TEXT_BRIGHT);
        label(
            "You are not here to win every auction. You are here to buy the right homes.",
            x + 2.0,
            150.0,
            19,
            TEXT_DIM,
        );

        let goal = Rect::new(x, 182.0, width, 102.0);
        highlight_panel(goal);
        label(
            "THE PORTFOLIO BRIEF",
            goal.x + 20.0,
            goal.y + 30.0,
            17,
            ACCENT,
        );
        label(
            &format!(
                "Own {} homes  +  earn {} rent each week  +  hold {} net worth",
                CAMPAIGN_GOAL_PROPERTIES,
                format_money(CAMPAIGN_GOAL_WEEKLY_RENT),
                format_money(CAMPAIGN_GOAL_NET_WORTH)
            ),
            goal.x + 20.0,
            goal.y + 62.0,
            26,
            TEXT_BRIGHT,
        );
        label(
            &format!("Build it before the end of week {CAMPAIGN_MAX_WEEKS}."),
            goal.x + 20.0,
            goal.y + 88.0,
            16,
            TEXT_DIM,
        );

        let cards_y = 308.0;
        let gap = 14.0;
        let card_w = (width - gap * 3.0) / 4.0;
        let cards = [
            (
                "1  RESEARCH",
                "Compare the guide, likely value, defects, rent, and your cash after settlement.",
                crate::ui::BLUE,
            ),
            (
                "2  SET YOUR LIMIT",
                "Choose a walk-away price before the auctioneer starts creating urgency.",
                POSITIVE,
            ),
            (
                "3  READ THE ROOM",
                "Raise, assert once, wait for a tell, or walk away. Winning can still be a mistake.",
                WARNING,
            ),
            (
                "4  MAKE IT WORK",
                "Lease for income, renovate for value, or sell to recycle your deposit into better homes.",
                ACCENT,
            ),
        ];
        for (index, (title, copy, color)) in cards.iter().enumerate() {
            let rect = Rect::new(x + index as f32 * (card_w + gap), cards_y, card_w, 170.0);
            soft_panel(rect);
            draw_badge(
                title,
                Rect::new(rect.x + 14.0, rect.y + 15.0, rect.w - 28.0, 28.0),
                *color,
            );
            draw_wrapped_text(copy, rect.x + 16.0, rect.y + 66.0, rect.w - 32.0, 17, TEXT);
        }

        let footer_y = ui_height() - 126.0;
        label(
            "You have two auction registrations each week. Using neither is allowed; patience preserves capital.",
            x,
            footer_y + 28.0,
            17,
            TEXT_DIM,
        );
        if button(
            Rect::new(x + width - 280.0, footer_y, 280.0, 54.0),
            "OPEN WEEK 1 LISTINGS",
            true,
            ButtonTone::Primary,
        ) {
            self.screen = Screen::PropertyList;
            self.status =
                "Tap INSPECT on a listing. Research it, then tap REGISTER TO BID.".to_string();
        }
    }
}
