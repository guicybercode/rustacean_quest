use crate::constants::*;
use macroquad::prelude::*;

pub struct Transition {
    pub timer: f32,
    pub alpha: f32,
    pub target_state: Option<crate::game::GameState>,
}

impl Transition {
    pub fn new() -> Self {
        Self {
            timer: 0.0,
            alpha: 0.0,
            target_state: None,
        }
    }

    pub fn start(&mut self, target: crate::game::GameState) {
        self.target_state = Some(target);
        self.timer = 0.0;
        self.alpha = 1.0;
    }

    pub fn update(&mut self, dt: f32) -> Option<crate::game::GameState> {
        if self.target_state.is_none() {
            return None;
        }

        if self.alpha <= 0.0 {
            return None;
        }

        self.timer += dt;
        let progress = (self.timer / TRANSITION_DURATION).min(1.0);
        let eased_progress = progress * progress * (3.0 - 2.0 * progress);
        self.alpha = 1.0 - eased_progress;

        if progress >= 1.0 {
            let target = self.target_state.take();
            self.timer = 0.0;
            self.alpha = 0.0;
            return target;
        }

        None
    }

    pub fn draw(&self) {
        if self.timer > 0.0 && self.alpha > 0.0 {
            let smooth_alpha = self.alpha * self.alpha;
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, smooth_alpha),
            );
        }
    }

    pub fn is_active(&self) -> bool {
        self.target_state.is_some() && self.alpha > 0.0
    }
}
