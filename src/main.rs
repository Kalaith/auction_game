#![allow(clippy::too_many_arguments)]

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod app;
mod data;
mod model;
mod save;
mod screens;
mod sim;
mod ui;

use app::App;

const UI_FONT_SIZES: &[u16] = &[
    12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 30, 31, 32, 34, 35, 38, 42,
    44, 74,
];

fn window_conf() -> Conf {
    // Hand-built Conf means no automatic arming: without this the capture run
    // puts a full game window on the desktop for its whole duration.
    capture::headless::arm("AUCTION_GAME");

    // Built by hand (not capture::capture_window_conf) to keep sample_count: 0;
    // high_dpi already defaults to false, so captures are pixel-aligned
    // regardless of capture mode.
    Conf {
        window_title: "Auction House Tycoon".to_owned(),
        window_width: capture::env_i32("AUCTION_GAME_WINDOW_WIDTH", 1280),
        window_height: capture::env_i32("AUCTION_GAME_WINDOW_HEIGHT", 720),
        window_resizable: true,
        high_dpi: false,
        sample_count: 0,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    macroquad_toolkit::ui::prewarm_default_ui_font(UI_FONT_SIZES)
        .expect("toolkit UI font should prewarm");
    let title_background =
        Texture2D::from_file_with_format(include_bytes!("../auction_house_title.png"), None);
    let mut app = App::new(title_background);

    // Present the populated font atlas before any dense UI frame is batched.
    // Macroquad otherwise lets the first screen share a frame with atlas uploads,
    // which can leave stale glyph coordinates after the texture grows.
    clear_background(crate::ui::BACKGROUND);
    macroquad_toolkit::ui::draw_default_ui_font_atlas_warmup(UI_FONT_SIZES);
    next_frame().await;

    // Screenshot harness: when AUCTION_GAME_CAPTURE_PATH is set, seed a scene,
    // simulate deterministic frames, write a PNG, and exit.
    if let Some(configs) = capture::CaptureConfig::all_from_env("AUCTION_GAME") {
        for config in configs {
            app.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |dt| {
                app.update(dt);
                app.draw();
            })
            .await;
        }
        return;
    }

    loop {
        let dt = get_frame_time();
        app.update(dt);
        app.draw();
        next_frame().await;
    }
}
