use macroquad::prelude::*;

mod audio;
mod camera;
mod checkpoint;
mod coin;
pub mod constants;
mod enemy;
mod game;
mod name_filter;
mod platform;
mod player;
mod save;

fn window_conf() -> Conf {
    Conf {
        window_title: "Jump Quest".to_owned(),
        window_width: constants::SCREEN_WIDTH as i32,
        window_height: constants::SCREEN_HEIGHT as i32,
        window_resizable: false,
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = game::Game::new().await;

    loop {
        let dt = get_frame_time();

        game.update(dt);
        game.draw();

        next_frame().await;
    }
}
