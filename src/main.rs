use macroquad::prelude::*;

mod app;
mod data;
mod model;
mod save;
mod screens;
mod sim;
mod ui;

use app::App;
use ui::set_ui_font;

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
    let font = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/DejaVuSans.ttf"))
        .expect("embedded UI font should load");
    set_ui_font(font);
    let title_background =
        Texture2D::from_file_with_format(include_bytes!("../auction_house_title.png"), None);
    let mut app = App::new(title_background);

    loop {
        let dt = get_frame_time();
        app.update(dt);
        app.draw();
        next_frame().await;
    }
}
