use crate::model::{Condition, Property};
use macroquad::prelude::*;

pub const BACKGROUND: Color = Color::new(0.075, 0.082, 0.088, 1.0);
pub const PANEL: Color = Color::new(0.138, 0.153, 0.157, 0.96);
pub const PANEL_DARK: Color = Color::new(0.092, 0.102, 0.110, 1.0);
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

pub fn button(rect: Rect, label: &str, enabled: bool, tone: ButtonTone) -> bool {
    let (mouse_x, mouse_y) = mouse_position();
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
    let font_size = if rect.w < 92.0 { 16 } else { 19 };
    draw_centered_text(label, rect, font_size, text_color);
    enabled && hovered && is_mouse_button_released(MouseButton::Left)
}

pub fn panel(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, PANEL_EDGE);
}

pub fn dark_panel(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_DARK);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, PANEL_EDGE);
}

pub fn label(text: &str, x: f32, y: f32, size: u16, color: Color) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font_size: size,
            color,
            ..Default::default()
        },
    );
}

pub fn draw_value(label_text: &str, value: &str, x: f32, y: f32, width: f32) {
    label(label_text, x, y, 16, TEXT_DIM);
    let measured = measure_text(value, None, 22, 1.0);
    label(value, x + width - measured.width, y, 22, TEXT_BRIGHT);
}

pub fn draw_wrapped_text(
    text: &str,
    x: f32,
    mut y: f32,
    max_width: f32,
    size: u16,
    color: Color,
) -> f32 {
    let mut line = String::new();
    let line_height = size as f32 + 7.0;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if measure_text(&candidate, None, size, 1.0).width > max_width && !line.is_empty() {
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
    label(label_text, rect.x, rect.y - 6.0, 15, TEXT_DIM);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_DARK);
    let fill = rect.w * (value as f32 / 100.0).clamp(0.0, 1.0);
    draw_rectangle(rect.x, rect.y, fill, rect.h, color);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, PANEL_EDGE);
}

pub fn draw_house_art(rect: Rect, property: &Property) {
    let sky = Color::new(0.290, 0.455, 0.565, 1.0);
    let grass = Color::new(0.225, 0.455, 0.295, 1.0);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, sky);
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

    let house_w = rect.w * 0.58;
    let house_h = rect.h * 0.36;
    let house_x = rect.x + rect.w * 0.22;
    let house_y = rect.y + rect.h * 0.46;
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

    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, PANEL_EDGE);
}

fn draw_centered_text(text: &str, rect: Rect, font_size: u16, color: Color) {
    let measured = measure_text(text, None, font_size, 1.0);
    draw_text_ex(
        text,
        rect.x + rect.w * 0.5 - measured.width * 0.5,
        rect.y + rect.h * 0.5 + measured.height * 0.36,
        TextParams {
            font_size,
            color,
            ..Default::default()
        },
    );
}
