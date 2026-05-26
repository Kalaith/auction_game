use crate::app::App;
use crate::ui::*;
use macroquad::prelude::*;

const SHADE: Color = Color::new(0.025, 0.028, 0.030, 0.40);

impl App {
    pub(crate) fn draw_title_screen(&mut self) {
        draw_title_background(&self.title_background);
        draw_rectangle(0.0, 0.0, ui_width(), ui_height(), SHADE);

        if self.title_settings_open {
            self.draw_title_settings();
        } else {
            self.draw_title_menu();
        }
    }

    fn draw_title_menu(&mut self) {
        let button_x = 92.0;
        let button_w = 328.0;
        let mut y = 318.0;

        if button(
            Rect::new(button_x, y, button_w, 48.0),
            "New Game",
            true,
            ButtonTone::Primary,
        ) {
            self.start_new_game();
        }
        y += 62.0;
        if button(
            Rect::new(button_x, y, button_w, 48.0),
            "Load Game",
            true,
            ButtonTone::Secondary,
        ) {
            self.load_game_from_title();
        }
        y += 62.0;
        if button(
            Rect::new(button_x, y, button_w, 48.0),
            "Settings",
            true,
            ButtonTone::Ghost,
        ) {
            self.title_settings_open = true;
        }
        y += 62.0;
        if button(
            Rect::new(button_x, y, button_w, 48.0),
            "Exit Game",
            true,
            ButtonTone::Danger,
        ) {
            macroquad::miniquad::window::quit();
        }
    }

    fn draw_title_settings(&mut self) {
        let button_x = 92.0;
        let button_w = 328.0;
        let fullscreen_label = if self.fullscreen_enabled {
            "Fullscreen: On"
        } else {
            "Fullscreen: Off"
        };

        if button(
            Rect::new(button_x, 386.0, button_w, 48.0),
            fullscreen_label,
            true,
            ButtonTone::Primary,
        ) {
            self.toggle_fullscreen();
        }

        if button(
            Rect::new(button_x, 448.0, button_w, 48.0),
            "Back",
            true,
            ButtonTone::Secondary,
        ) {
            self.title_settings_open = false;
        }
    }
}

fn draw_title_background(texture: &Texture2D) {
    let texture_size = texture.size();
    let scale = (ui_width() / texture_size.x).max(ui_height() / texture_size.y);
    let dest_size = vec2(texture_size.x * scale, texture_size.y * scale);
    let x = (ui_width() - dest_size.x) * 0.5;
    let y = (ui_height() - dest_size.y) * 0.5;

    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(dest_size),
            ..Default::default()
        },
    );
}
