use crate::constants::*;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub lifetime: f32,
}

impl Particle {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            lifetime: PARTICLE_LIFETIME,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.lifetime -= dt;
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32, colorblind_mode: bool) {
        if !self.is_alive() {
            return;
        }

        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        let lifetime_ratio = self.lifetime / PARTICLE_LIFETIME;
        let alpha = (lifetime_ratio * lifetime_ratio).max(0.0).min(1.0);
        let radius = PARTICLE_MIN_RADIUS
            + (PARTICLE_MAX_RADIUS - PARTICLE_MIN_RADIUS) * (1.0 - lifetime_ratio);
        let color_variation = (self.x + self.y) as u32 % 3;
        let color = if colorblind_mode {
            Color::new(0.5, 0.5, 0.5, alpha)
        } else {
            match color_variation {
                0 => Color::new(1.0, 0.84, 0.0, alpha),
                1 => Color::new(1.0, 0.9, 0.3, alpha),
                _ => Color::new(1.0, 0.7, 0.0, alpha),
            }
        };
        draw_circle(screen_x, screen_y, radius, color);
    }
}

#[derive(Clone, Copy)]
pub struct CoinBounce {
    pub x: f32,
    pub y: f32,
    pub timer: f32,
}

impl CoinBounce {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            timer: COIN_BOUNCE_DURATION,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timer -= dt;
    }

    pub fn is_alive(&self) -> bool {
        self.timer > 0.0
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32, colorblind_mode: bool) {
        if !self.is_alive() {
            return;
        }

        let progress = 1.0 - (self.timer / COIN_BOUNCE_DURATION);
        let scale = 1.0 + (COIN_BOUNCE_SCALE - 1.0) * (1.0 - (progress * 2.0 - 1.0).abs());
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        let alpha = (1.0 - progress).max(0.0);
        let color = if colorblind_mode {
            Color::new(0.5, 0.5, 0.5, alpha)
        } else {
            Color::new(1.0, 0.84, 0.0, alpha)
        };
        draw_circle(screen_x, screen_y, (COIN_SIZE / 2.0) * scale, color);
        draw_circle_lines(screen_x, screen_y, (COIN_SIZE / 2.0) * scale, 2.0, color);
    }
}
