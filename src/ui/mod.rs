use crate::model::{Condition, Property};
use macroquad::prelude::*;
use std::cell::RefCell;

thread_local! {
    static UI_FONT: RefCell<Option<Font>> = const { RefCell::new(None) };
}

pub const UI_WIDTH: f32 = 1200.0;
pub const UI_HEIGHT: f32 = 675.0;
pub const BACKGROUND: Color = Color::new(0.075, 0.082, 0.088, 1.0);
pub const PANEL_DARK: Color = Color::new(0.092, 0.102, 0.110, 1.0);
pub const PANEL_SOFT: Color = Color::new(0.105, 0.119, 0.126, 0.92);
pub const PANEL_EDGE: Color = Color::new(0.302, 0.337, 0.329, 1.0);
pub const TEXT: Color = Color::new(0.890, 0.902, 0.878, 1.0);
pub const TEXT_DIM: Color = Color::new(0.620, 0.655, 0.635, 1.0);
pub const TEXT_BRIGHT: Color = Color::new(0.985, 0.975, 0.925, 1.0);
pub const ACCENT: Color = Color::new(0.980, 0.690, 0.280, 1.0);
pub const POSITIVE: Color = Color::new(0.360, 0.760, 0.545, 1.0);
pub const WARNING: Color = Color::new(0.930, 0.735, 0.300, 1.0);
pub const NEGATIVE: Color = Color::new(0.870, 0.345, 0.320, 1.0);
pub const BLUE: Color = Color::new(0.325, 0.570, 0.750, 1.0);

#[derive(Clone, Copy)]
pub enum ButtonTone {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

pub fn set_ui_font(font: Font) {
    UI_FONT.with(|stored| {
        *stored.borrow_mut() = Some(font);
    });
}

pub fn begin_ui_frame() {
    set_camera(&Camera2D {
        target: vec2(UI_WIDTH * 0.5, UI_HEIGHT * 0.5),
        zoom: vec2(2.0 / UI_WIDTH, 2.0 / UI_HEIGHT),
        ..Default::default()
    });
}

pub fn ui_width() -> f32 {
    UI_WIDTH
}

pub fn ui_height() -> f32 {
    UI_HEIGHT
}

fn mouse_position_ui() -> (f32, f32) {
    let (x, y) = mouse_position();
    (
        x * UI_WIDTH / screen_width().max(1.0),
        y * UI_HEIGHT / screen_height().max(1.0),
    )
}

pub fn format_money(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let mut digits = value.abs().to_string();
    let mut result = String::new();
    while digits.len() > 3 {
        let tail = digits.split_off(digits.len() - 3);
        if result.is_empty() {
            result = tail;
        } else {
            result = format!("{tail},{result}");
        }
    }
    if result.is_empty() {
        format!("{sign}${digits}")
    } else {
        format!("{sign}${digits},{result}")
    }
}

pub fn format_compact_money(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let abs = value.abs();
    if abs >= 1_000_000 {
        format!("{sign}${:.1}m", abs as f32 / 1_000_000.0)
    } else if abs >= 1_000 {
        format!("{sign}${}k", abs / 1_000)
    } else {
        format!("{sign}${abs}")
    }
}

pub fn button(rect: Rect, label: &str, enabled: bool, tone: ButtonTone) -> bool {
    let (mouse_x, mouse_y) = mouse_position_ui();
    let hovered = enabled
        && mouse_x >= rect.x
        && mouse_x <= rect.x + rect.w
        && mouse_y >= rect.y
        && mouse_y <= rect.y + rect.h;

    let base = match tone {
        ButtonTone::Primary => ACCENT,
        ButtonTone::Secondary => BLUE,
        ButtonTone::Danger => NEGATIVE,
        ButtonTone::Ghost => PANEL_DARK,
    };
    let mut color = if enabled {
        base
    } else {
        Color::new(0.18, 0.19, 0.19, 1.0)
    };
    if hovered {
        color = Color::new(
            (color.r + 0.08).min(1.0),
            (color.g + 0.08).min(1.0),
            (color.b + 0.08).min(1.0),
            1.0,
        );
    }

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, PANEL_EDGE);

    let text_color = if enabled { TEXT_BRIGHT } else { TEXT_DIM };
    let font_size = if rect.w < 92.0 { 18 } else { 20 };
    draw_centered_text(label, rect, font_size, text_color);
    enabled && hovered && is_mouse_button_released(MouseButton::Left)
}

pub fn dark_panel(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_DARK);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, PANEL_EDGE);
}

pub fn soft_panel(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_SOFT);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::new(PANEL_EDGE.r, PANEL_EDGE.g, PANEL_EDGE.b, 0.58),
    );
}

pub fn highlight_panel(rect: Rect) {
    soft_panel(rect);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, ACCENT);
}

pub fn label(text: &str, x: f32, y: f32, size: u16, color: Color) {
    let readable_size = size.max(17);
    UI_FONT.with(|stored| {
        let font = stored.borrow();
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: font.as_ref(),
                font_size: readable_size,
                color,
                ..Default::default()
            },
        );
    });
}

pub fn measure_label(text: &str, size: u16) -> TextDimensions {
    let readable_size = size.max(17);
    measure_text_raw(text, readable_size)
}

fn measure_text_raw(text: &str, size: u16) -> TextDimensions {
    UI_FONT.with(|stored| {
        let font = stored.borrow();
        measure_text(text, font.as_ref(), size, 1.0)
    })
}

pub fn draw_value(label_text: &str, value: &str, x: f32, y: f32, width: f32) {
    label(label_text, x, y, 17, TEXT_DIM);
    let measured = measure_label(value, 23);
    label(value, x + width - measured.width, y, 23, TEXT_BRIGHT);
}

pub fn draw_money_stat(label_text: &str, value: &str, note: &str, rect: Rect, color: Color) {
    soft_panel(rect);
    label(label_text, rect.x + 14.0, rect.y + 25.0, 17, TEXT_DIM);
    let label_width = measure_label(label_text, 17).width;
    let value_size = if rect.w < 230.0 { 25 } else { 27 };
    let measured = measure_label(value, value_size);
    let stacks_value = label_width + measured.width + 42.0 > rect.w;
    let value_x = if stacks_value {
        rect.x + 14.0
    } else {
        rect.x + rect.w - measured.width - 14.0
    };
    let value_y = if stacks_value {
        rect.y + 50.0
    } else {
        rect.y + 32.0
    };
    label(value, value_x, value_y, value_size, color);
    if !note.is_empty() {
        label(note, rect.x + 14.0, rect.y + rect.h - 12.0, 15, TEXT_DIM);
    }
}

pub fn draw_badge(text: &str, rect: Rect, color: Color) {
    let fill = Color::new(color.r * 0.18, color.g * 0.18, color.b * 0.18, 1.0);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, color);
    let mut size = 17;
    while size > 13 && measure_text_raw(text, size).width > rect.w - 8.0 {
        size -= 1;
    }
    draw_centered_text(text, rect, size, color);
}

pub fn rect_clicked(rect: Rect) -> bool {
    let (mouse_x, mouse_y) = mouse_position_ui();
    mouse_x >= rect.x
        && mouse_x <= rect.x + rect.w
        && mouse_y >= rect.y
        && mouse_y <= rect.y + rect.h
        && is_mouse_button_released(MouseButton::Left)
}

pub fn draw_centered_label(text: &str, rect: Rect, font_size: u16, color: Color) {
    draw_centered_text(text, rect, font_size.max(17), color);
}

pub fn label_fit(text: &str, x: f32, y: f32, max_width: f32, size: u16, color: Color) {
    let size = size.max(17);
    if measure_label(text, size).width <= max_width {
        label(text, x, y, size, color);
        return;
    }

    let mut clipped = text.to_string();
    while clipped.len() > 4 {
        clipped.pop();
        let candidate = format!("{}...", clipped.trim_end());
        if measure_label(&candidate, size).width <= max_width {
            label(&candidate, x, y, size, color);
            return;
        }
    }

    label(text, x, y, size, color);
}

pub fn draw_wrapped_text(
    text: &str,
    x: f32,
    mut y: f32,
    max_width: f32,
    size: u16,
    color: Color,
) -> f32 {
    let size = size.max(17);
    let mut line = String::new();
    let line_height = size as f32 + 8.0;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if measure_label(&candidate, size).width > max_width && !line.is_empty() {
            label(&line, x, y, size, color);
            y += line_height;
            line = word.to_string();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        label(&line, x, y, size, color);
        y += line_height;
    }
    y
}

pub fn draw_meter(label_text: &str, value: i32, rect: Rect, color: Color) {
    label(label_text, rect.x, rect.y - 6.0, 17, TEXT_DIM);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_DARK);
    let fill = rect.w * (value as f32 / 100.0).clamp(0.0, 1.0);
    draw_rectangle(rect.x, rect.y, fill, rect.h, color);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, PANEL_EDGE);
}

pub fn draw_house_art(rect: Rect, property: &Property) {
    let sky = Color::new(0.255, 0.430, 0.555, 1.0);
    let hills = Color::new(0.180, 0.330, 0.440, 1.0);
    let grass = Color::new(0.205, 0.445, 0.265, 1.0);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, sky);
    draw_triangle(
        vec2(rect.x, rect.y + rect.h * 0.66),
        vec2(rect.x + rect.w * 0.28, rect.y + rect.h * 0.42),
        vec2(rect.x + rect.w * 0.58, rect.y + rect.h * 0.66),
        hills,
    );
    draw_triangle(
        vec2(rect.x + rect.w * 0.32, rect.y + rect.h * 0.66),
        vec2(rect.x + rect.w * 0.72, rect.y + rect.h * 0.36),
        vec2(rect.x + rect.w, rect.y + rect.h * 0.66),
        Color::new(0.155, 0.305, 0.395, 1.0),
    );
    draw_cloud(
        rect.x + rect.w * 0.10,
        rect.y + rect.h * 0.20,
        rect.w * 0.18,
    );
    draw_cloud(
        rect.x + rect.w * 0.70,
        rect.y + rect.h * 0.16,
        rect.w * 0.14,
    );
    draw_rectangle(rect.x, rect.y + rect.h * 0.70, rect.w, rect.h * 0.30, grass);

    let palette = match property.id % 5 {
        0 => (
            Color::new(0.650, 0.315, 0.245, 1.0),
            Color::new(0.280, 0.170, 0.140, 1.0),
        ),
        1 => (
            Color::new(0.725, 0.640, 0.470, 1.0),
            Color::new(0.245, 0.250, 0.235, 1.0),
        ),
        2 => (
            Color::new(0.600, 0.675, 0.650, 1.0),
            Color::new(0.130, 0.220, 0.275, 1.0),
        ),
        3 => (
            Color::new(0.500, 0.440, 0.360, 1.0),
            Color::new(0.300, 0.235, 0.175, 1.0),
        ),
        _ => (
            Color::new(0.690, 0.720, 0.680, 1.0),
            Color::new(0.275, 0.230, 0.220, 1.0),
        ),
    };

    let house_w = rect.w * 0.60;
    let house_h = rect.h * 0.38;
    let house_x = rect.x + rect.w * 0.20;
    let house_y = rect.y + rect.h * 0.47;
    draw_ellipse(
        house_x + house_w * 0.48,
        house_y + house_h + rect.h * 0.04,
        house_w * 0.58,
        rect.h * 0.035,
        0.0,
        Color::new(0.050, 0.070, 0.060, 0.28),
    );
    draw_rectangle(house_x, house_y, house_w, house_h, palette.0);
    draw_triangle(
        vec2(house_x - 12.0, house_y),
        vec2(house_x + house_w * 0.5, house_y - rect.h * 0.22),
        vec2(house_x + house_w + 12.0, house_y),
        palette.1,
    );

    let window_color = Color::new(0.850, 0.860, 0.725, 1.0);
    draw_rectangle(
        house_x + house_w * 0.12,
        house_y + house_h * 0.24,
        house_w * 0.18,
        house_h * 0.22,
        window_color,
    );
    draw_rectangle(
        house_x + house_w * 0.70,
        house_y + house_h * 0.24,
        house_w * 0.18,
        house_h * 0.22,
        window_color,
    );
    draw_rectangle(
        house_x + house_w * 0.43,
        house_y + house_h * 0.42,
        house_w * 0.16,
        house_h * 0.58,
        palette.1,
    );
    draw_line(
        house_x + house_w * 0.51,
        house_y + house_h,
        house_x + house_w * 0.46,
        rect.y + rect.h,
        rect.w * 0.025,
        Color::new(0.205, 0.185, 0.155, 1.0),
    );

    if matches!(property.condition, Condition::Rough | Condition::Tired) {
        draw_line(
            house_x + 12.0,
            house_y + house_h - 12.0,
            house_x + 42.0,
            house_y + house_h - 20.0,
            3.0,
            PANEL_EDGE,
        );
        draw_line(
            house_x + house_w - 18.0,
            house_y + 8.0,
            house_x + house_w - 42.0,
            house_y + 30.0,
            2.5,
            PANEL_EDGE,
        );
    }

    for offset in [0.08, 0.88] {
        draw_circle(
            rect.x + rect.w * offset,
            rect.y + rect.h * 0.76,
            rect.h * 0.045,
            Color::new(0.115, 0.345, 0.170, 1.0),
        );
    }

    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, PANEL_EDGE);
}

fn draw_cloud(x: f32, y: f32, scale: f32) {
    let color = Color::new(0.820, 0.850, 0.805, 0.88);
    draw_circle(x, y + scale * 0.22, scale * 0.18, color);
    draw_circle(x + scale * 0.18, y + scale * 0.12, scale * 0.23, color);
    draw_circle(x + scale * 0.40, y + scale * 0.20, scale * 0.17, color);
    draw_rectangle(
        x - scale * 0.06,
        y + scale * 0.20,
        scale * 0.56,
        scale * 0.15,
        color,
    );
}

fn draw_centered_text(text: &str, rect: Rect, font_size: u16, color: Color) {
    let font_size = font_size.max(12);
    let measured = measure_text_raw(text, font_size);
    UI_FONT.with(|stored| {
        let font = stored.borrow();
        draw_text_ex(
            text,
            rect.x + rect.w * 0.5 - measured.width * 0.5,
            rect.y + rect.h * 0.5 + measured.height * 0.36,
            TextParams {
                font: font.as_ref(),
                font_size,
                color,
                ..Default::default()
            },
        );
    });
}
