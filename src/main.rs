use macroquad::prelude::*;

mod app;
mod data;
mod model;
mod screens;
mod sim;
mod ui;

use app::App;

fn window_conf() -> Conf {
    Conf {
        window_title: "Auction House Tycoon".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        high_dpi: false,
        sample_count: 0,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new();

    loop {
        let dt = get_frame_time();
        app.update(dt);
        app.draw();
        next_frame().await;
    }
}
