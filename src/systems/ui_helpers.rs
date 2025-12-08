use crate::constants::*;
use macroquad::prelude::*;

pub struct MenuAnimation {
    animations: Vec<f32>,
}

impl MenuAnimation {
    pub fn new(count: usize) -> Self {
        Self {
            animations: vec![0.0; count],
        }
    }

    pub fn trigger(&mut self, index: usize) {
        if index < self.animations.len() {
            self.animations[index] = MENU_BOUNCE_DURATION;
        }
    }

    pub fn update(&mut self, dt: f32) {
        for anim in &mut self.animations {
            *anim = (*anim - dt * 8.0).max(0.0);
        }
    }

    pub fn get_scale(&self, index: usize) -> f32 {
        if index >= self.animations.len() {
            return 1.0;
        }

        let bounce_progress = self.animations[index] / MENU_BOUNCE_DURATION;
        if bounce_progress > 0.0 {
            1.0 + (MENU_BOUNCE_SCALE - 1.0) * (1.0 - (bounce_progress * 2.0 - 1.0).abs())
        } else {
            1.0
        }
    }
}
