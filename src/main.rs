use macroquad::prelude::*;

mod player;
mod enemy;
mod platform;
mod coin;
mod camera;
mod audio;
mod checkpoint;
mod game;
mod save;
mod name_filter;
pub mod constants;

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
