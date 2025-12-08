use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct Platform {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Platform {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    #[inline]
    pub fn check_collision(&self, other_x: f32, other_y: f32, other_w: f32, other_h: f32) -> bool {
        other_x < self.x + self.width
            && other_x + other_w > self.x
            && other_y < self.y + self.height
            && other_y + other_h > self.y
    }

    pub fn get_collision_response(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        vel_y: f32,
    ) -> Option<(f32, f32, bool)> {
        if !self.check_collision(x, y, w, h) {
            return None;
        }
        
        let overlap_left = (x + w) - self.x;
        let overlap_right = (self.x + self.width) - x;
        let overlap_top = (y + h) - self.y;
        let overlap_bottom = (self.y + self.height) - y;
        
        let min_overlap = overlap_left.min(overlap_right).min(overlap_top).min(overlap_bottom);
        
        if min_overlap == overlap_top && vel_y >= 0.0 {
            return Some((x, self.y - h, true));
        }
        
        if min_overlap == overlap_bottom && vel_y < 0.0 {
            return Some((x, self.y + self.height, false));
        }
        
        if min_overlap == overlap_left {
            return Some((self.x - w, y, false));
        }
        if min_overlap == overlap_right {
            return Some((self.x + self.width, y, false));
        }
        
        if overlap_top < overlap_bottom {
            Some((x, self.y - h, true))
        } else {
            None
        }
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        
        draw_rectangle(screen_x, screen_y, self.width, self.height, WHITE);
        draw_rectangle_lines(screen_x, screen_y, self.width, self.height, 2.0, BLACK);
    }
}

pub fn create_level_platforms(level: usize) -> Vec<Platform> {
    let mut platforms = Vec::with_capacity(crate::constants::ESTIMATED_PLATFORMS_PER_LEVEL);
    
    match level {
        1 => {
            platforms.push(Platform::new(0.0, 550.0, 4200.0, 50.0));
            platforms.push(Platform::new(200.0, 450.0, 150.0, 20.0));
            platforms.push(Platform::new(400.0, 400.0, 150.0, 20.0));
            platforms.push(Platform::new(600.0, 350.0, 150.0, 20.0));
            platforms.push(Platform::new(800.0, 450.0, 200.0, 20.0));
            platforms.push(Platform::new(1100.0, 400.0, 150.0, 20.0));
            platforms.push(Platform::new(1300.0, 350.0, 150.0, 20.0));
            platforms.push(Platform::new(1500.0, 400.0, 150.0, 20.0));
            platforms.push(Platform::new(1700.0, 300.0, 200.0, 20.0));
            platforms.push(Platform::new(2000.0, 450.0, 150.0, 20.0));
            platforms.push(Platform::new(2200.0, 400.0, 150.0, 20.0));
            platforms.push(Platform::new(2400.0, 350.0, 150.0, 20.0));
            platforms.push(Platform::new(500.0, 500.0, 40.0, 50.0));
            platforms.push(Platform::new(1200.0, 500.0, 40.0, 50.0));
            platforms.push(Platform::new(1900.0, 500.0, 40.0, 50.0));
        }
        2 => {
            platforms.push(Platform::new(0.0, 550.0, 4200.0, 50.0));
            platforms.push(Platform::new(150.0, 500.0, 100.0, 20.0));
            platforms.push(Platform::new(300.0, 450.0, 100.0, 20.0));
            platforms.push(Platform::new(450.0, 400.0, 100.0, 20.0));
            platforms.push(Platform::new(600.0, 350.0, 100.0, 20.0));
            platforms.push(Platform::new(750.0, 300.0, 100.0, 20.0));
            platforms.push(Platform::new(900.0, 250.0, 100.0, 20.0));
            platforms.push(Platform::new(1050.0, 300.0, 100.0, 20.0));
            platforms.push(Platform::new(1200.0, 350.0, 100.0, 20.0));
            platforms.push(Platform::new(1350.0, 400.0, 100.0, 20.0));
            platforms.push(Platform::new(1500.0, 450.0, 100.0, 20.0));
            platforms.push(Platform::new(1650.0, 500.0, 100.0, 20.0));
            platforms.push(Platform::new(1800.0, 400.0, 200.0, 20.0));
            platforms.push(Platform::new(2100.0, 350.0, 150.0, 20.0));
            platforms.push(Platform::new(2300.0, 300.0, 150.0, 20.0));
            platforms.push(Platform::new(2500.0, 250.0, 200.0, 20.0));
        }
        3 => {
            platforms.push(Platform::new(0.0, 550.0, 4200.0, 50.0));
            platforms.push(Platform::new(100.0, 450.0, 200.0, 20.0));
            platforms.push(Platform::new(400.0, 500.0, 150.0, 20.0));
            platforms.push(Platform::new(650.0, 450.0, 100.0, 20.0));
            platforms.push(Platform::new(850.0, 400.0, 150.0, 20.0));
            platforms.push(Platform::new(1100.0, 350.0, 100.0, 20.0));
            platforms.push(Platform::new(1300.0, 300.0, 200.0, 20.0));
            platforms.push(Platform::new(1600.0, 350.0, 150.0, 20.0));
            platforms.push(Platform::new(1850.0, 400.0, 100.0, 20.0));
            platforms.push(Platform::new(2050.0, 450.0, 150.0, 20.0));
            platforms.push(Platform::new(2300.0, 500.0, 100.0, 20.0));
            platforms.push(Platform::new(2500.0, 450.0, 200.0, 20.0));
            platforms.push(Platform::new(300.0, 300.0, 80.0, 20.0));
            platforms.push(Platform::new(500.0, 250.0, 80.0, 20.0));
            platforms.push(Platform::new(700.0, 200.0, 80.0, 20.0));
            platforms.push(Platform::new(1500.0, 200.0, 80.0, 20.0));
            platforms.push(Platform::new(1700.0, 250.0, 80.0, 20.0));
        }
        4 => {
            platforms.push(Platform::new(0.0, 550.0, 4200.0, 50.0));
            platforms.push(Platform::new(100.0, 500.0, 120.0, 20.0));
            platforms.push(Platform::new(250.0, 450.0, 120.0, 20.0));
            platforms.push(Platform::new(400.0, 500.0, 120.0, 20.0));
            platforms.push(Platform::new(550.0, 450.0, 120.0, 20.0));
            platforms.push(Platform::new(700.0, 500.0, 120.0, 20.0));
            platforms.push(Platform::new(850.0, 450.0, 120.0, 20.0));
            platforms.push(Platform::new(1000.0, 350.0, 150.0, 20.0));
            platforms.push(Platform::new(1200.0, 300.0, 150.0, 20.0));
            platforms.push(Platform::new(1400.0, 250.0, 150.0, 20.0));
            platforms.push(Platform::new(1600.0, 200.0, 150.0, 20.0));
            platforms.push(Platform::new(1800.0, 400.0, 100.0, 20.0));
            platforms.push(Platform::new(1950.0, 350.0, 100.0, 20.0));
            platforms.push(Platform::new(2100.0, 300.0, 100.0, 20.0));
            platforms.push(Platform::new(2250.0, 250.0, 100.0, 20.0));
            platforms.push(Platform::new(2400.0, 400.0, 80.0, 20.0));
            platforms.push(Platform::new(2550.0, 350.0, 80.0, 20.0));
            platforms.push(Platform::new(2700.0, 300.0, 80.0, 20.0));
            platforms.push(Platform::new(600.0, 500.0, 40.0, 50.0));
            platforms.push(Platform::new(1500.0, 500.0, 40.0, 50.0));
            platforms.push(Platform::new(2300.0, 500.0, 40.0, 50.0));
        }
        5 => {
            platforms.push(Platform::new(0.0, 550.0, 4200.0, 50.0));

            platforms.push(Platform::new(180.0, 500.0, 120.0, 20.0));
            platforms.push(Platform::new(420.0, 440.0, 120.0, 20.0));
            platforms.push(Platform::new(680.0, 380.0, 110.0, 20.0));
            platforms.push(Platform::new(950.0, 330.0, 100.0, 20.0));

            platforms.push(Platform::new(1250.0, 420.0, 180.0, 20.0));
            platforms.push(Platform::new(1500.0, 320.0, 140.0, 20.0));
            platforms.push(Platform::new(1700.0, 260.0, 120.0, 20.0));
            platforms.push(Platform::new(1900.0, 210.0, 100.0, 20.0));

            platforms.push(Platform::new(2100.0, 440.0, 180.0, 20.0));
            platforms.push(Platform::new(2350.0, 380.0, 140.0, 20.0));
            platforms.push(Platform::new(2550.0, 320.0, 120.0, 20.0));
            platforms.push(Platform::new(2750.0, 260.0, 100.0, 20.0));

            platforms.push(Platform::new(3000.0, 500.0, 140.0, 20.0));
            platforms.push(Platform::new(3200.0, 440.0, 120.0, 20.0));
            platforms.push(Platform::new(3400.0, 380.0, 100.0, 20.0));
            platforms.push(Platform::new(3600.0, 320.0, 90.0, 20.0));

            platforms.push(Platform::new(3800.0, 260.0, 140.0, 20.0));
            platforms.push(Platform::new(4020.0, 220.0, 140.0, 20.0));

            platforms.push(Platform::new(1700.0, 140.0, 80.0, 20.0));
            platforms.push(Platform::new(2600.0, 160.0, 80.0, 20.0));
            platforms.push(Platform::new(3200.0, 200.0, 70.0, 20.0));

            platforms.push(Platform::new(900.0, 500.0, 40.0, 50.0));
            platforms.push(Platform::new(1900.0, 500.0, 40.0, 50.0));
            platforms.push(Platform::new(3100.0, 500.0, 40.0, 50.0));
            platforms.push(Platform::new(3800.0, 500.0, 40.0, 50.0));
        }
        _ => {
            platforms.push(Platform::new(0.0, 550.0, 4200.0, 50.0));
        }
    }
    
    platforms
}
