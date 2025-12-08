use crate::constants::*;

pub struct Camera {
    pub x: f32,
    pub y: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn update(&mut self, player_x: f32, screen_width: f32, shake_offset: f32) {
        let target_x = player_x - screen_width / 2.0;
        
        let min_x = 0.0;
        let max_x = WORLD_WIDTH - screen_width;
        
        self.x = (target_x + shake_offset).clamp(min_x, max_x);
    }
}
