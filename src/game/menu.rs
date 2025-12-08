use crate::constants::*;
use macroquad::prelude::*;

use super::{Game, GameState};

impl Game {
    pub fn update_menu_states(&mut self, dt: f32) {
        match self.state {
            GameState::Splash => {
                self.splash_timer += dt;
                if self.splash_timer >= SPLASH_DURATION {
                    self.state = GameState::Menu;
                    self.splash_timer = 0.0;
                    self.splash_shown = true;
                }
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Escape)
                {
                    self.state = GameState::Menu;
                    self.splash_timer = 0.0;
                    self.splash_shown = true;
                }
            }
            _ => {}
        }
    }
}

