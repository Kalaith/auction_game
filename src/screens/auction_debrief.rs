use crate::app::PurchaseDebrief;
use crate::ui::*;
use macroquad::prelude::*;

pub(super) fn draw_purchase_debrief(debrief: &PurchaseDebrief, rect: Rect) {
    label("You Won", rect.x + 26.0, rect.y + 44.0, 32, TEXT_BRIGHT);
    label(&debrief.address, rect.x + 30.0, rect.y + 70.0, 17, TEXT_DIM);
    label(
        &format_money(debrief.purchase_price),
        rect.x + 26.0,
        rect.y + 118.0,
        44,
        ACCENT,
    );
    label(
        &format!(
            "Estimated resale {}",
            format_money(debrief.estimated_resale)
        ),
        rect.x + 28.0,
        rect.y + 144.0,
        15,
        TEXT_DIM,
    );
    let values = [
        ("10% contract deposit", debrief.contract_deposit),
        ("Cash after settle", debrief.cash_after_settle),
        ("New property loan", debrief.loan_amount),
        ("Purchase + future sale fees", debrief.fees),
        ("Repair allowance", debrief.renovation_allowance),
        ("Rent appraisal / week", debrief.weekly_rent),
        ("Rental cashflow / week", debrief.weekly_rental_cashflow),
        ("Projected profit", debrief.projected_profit),
    ];
    for (index, (title, value)) in values.iter().enumerate() {
        draw_value(
            title,
            &format_money(*value),
            rect.x + 28.0,
            rect.y + 174.0 + index as f32 * 23.0,
            rect.w - 56.0,
        );
    }
    if debrief.walkaway_delta > 0 {
        label(
            &format!("Over walk-away by {}", format_money(debrief.walkaway_delta)),
            rect.x + 28.0,
            rect.y + 350.0,
            15,
            NEGATIVE,
        );
    }
    let lesson = Rect::new(rect.x + 24.0, rect.y + 366.0, rect.w - 48.0, 86.0);
    dark_panel(lesson);
    label("Lesson", lesson.x + 14.0, lesson.y + 28.0, 20, TEXT_BRIGHT);
    draw_wrapped_text(
        &debrief.lesson,
        lesson.x + 14.0,
        lesson.y + 56.0,
        lesson.w - 28.0,
        16,
        TEXT,
    );
}
