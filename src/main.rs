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
    macroquad_toolkit::ui::ensure_default_ui_font().expect("toolkit UI font should load");
    let title_background =
        Texture2D::from_file_with_format(include_bytes!("../auction_house_title.png"), None);
    let mut app = App::new(title_background);

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
