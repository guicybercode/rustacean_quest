use macroquad::prelude::*;
use crate::constants::*;

pub struct Checkpoint {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub activated: bool,
}

impl Checkpoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            width: CHECKPOINT_WIDTH,
            height: CHECKPOINT_HEIGHT,
            activated: false,
        }
    }

    pub fn check_activation(&mut self, player_x: f32, player_y: f32, player_w: f32, player_h: f32) -> bool {
        if !self.activated 
            && player_x < self.x + self.width
            && player_x + player_w > self.x
            && player_y < self.y + self.height
            && player_y + player_h > self.y
        {
            self.activated = true;
            return true;
        }
        false
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        
        if self.activated {
            draw_rectangle(screen_x, screen_y, self.width, self.height, DARKGREEN);
            draw_rectangle(screen_x + self.width - 10.0, screen_y, 10.0, 20.0, GREEN);
        } else {
            draw_rectangle(screen_x, screen_y, self.width, self.height, GRAY);
            draw_rectangle(screen_x + self.width - 10.0, screen_y, 10.0, 20.0, DARKGRAY);
        }
    }
}

pub fn create_level_checkpoints(level: usize) -> Vec<Checkpoint> {
    let mut checkpoints = Vec::with_capacity(crate::constants::ESTIMATED_CHECKPOINTS_PER_LEVEL);
    
    match level {
        1 => {
            checkpoints.push(Checkpoint::new(500.0, 490.0));
            checkpoints.push(Checkpoint::new(1200.0, 490.0));
            checkpoints.push(Checkpoint::new(2000.0, 490.0));
        }
        2 => {
            checkpoints.push(Checkpoint::new(600.0, 490.0));
            checkpoints.push(Checkpoint::new(1400.0, 490.0));
            checkpoints.push(Checkpoint::new(2200.0, 490.0));
        }
        3 => {
            checkpoints.push(Checkpoint::new(700.0, 490.0));
            checkpoints.push(Checkpoint::new(1600.0, 490.0));
            checkpoints.push(Checkpoint::new(2400.0, 490.0));
        }
        4 => {
            checkpoints.push(Checkpoint::new(800.0, 490.0));
            checkpoints.push(Checkpoint::new(1800.0, 490.0));
            checkpoints.push(Checkpoint::new(2600.0, 490.0));
        }
        5 => {
            checkpoints.push(Checkpoint::new(1400.0, 490.0));
            checkpoints.push(Checkpoint::new(3000.0, 490.0));
        }
        _ => {}
    }
    
    checkpoints
}
