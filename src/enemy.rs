use crate::constants::*;
use macroquad::prelude::*;
use std::rc::Rc;

pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub alive: bool,
    pub on_ground: bool,
    pub facing_right: bool,
    pub texture: Option<Rc<Texture2D>>,
    pub anim_frame: usize,
    pub anim_timer: f32,
}

impl Enemy {
    pub fn new(x: f32, y: f32, texture: Option<Rc<Texture2D>>) -> Self {
        Self {
            x,
            y,
            width: ENEMY_WIDTH,
            height: ENEMY_HEIGHT,
            vel_x: -ENEMY_SPEED,
            vel_y: 0.0,
            alive: true,
            on_ground: true,
            facing_right: false,
            texture,
            anim_frame: 0,
            anim_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.alive {
            return;
        }

        self.x += self.vel_x * dt;

        if !self.on_ground {
            self.vel_y += ENEMY_GRAVITY * dt;
        }

        if self.vel_y > TERMINAL_VELOCITY {
            self.vel_y = TERMINAL_VELOCITY;
        }

        self.y += self.vel_y * dt;

        self.on_ground = false;

        self.facing_right = self.vel_x >= 0.0;

        if self.texture.is_some() && self.on_ground {
            self.anim_timer += dt;
            if self.anim_timer >= crate::constants::ENEMY_ANIMATION_SPEED {
                self.anim_timer = 0.0;
                self.anim_frame = (self.anim_frame + 1) % 4;
            }
        } else {
            self.anim_frame = 0;
            self.anim_timer = 0.0;
        }
    }

    #[inline]
    pub fn check_platform_collision(&mut self, platform: &crate::platform::Platform) {
        if !self.alive {
            return;
        }

        if self.x < platform.x + platform.width
            && self.x + self.width > platform.x
            && self.y < platform.y + platform.height
            && self.y + self.height > platform.y
        {
            let overlap_top = (self.y + self.height) - platform.y;
            let overlap_left = (self.x + self.width) - platform.x;
            let overlap_right = (platform.x + platform.width) - self.x;

            if overlap_top < overlap_left.min(overlap_right)
                && overlap_top < PLATFORM_COLLISION_THRESHOLD
                && self.vel_y >= 0.0
            {
                self.y = platform.y - self.height;
                self.vel_y = 0.0;
                self.on_ground = true;
            } else {
                self.vel_x = -self.vel_x;
                if self.vel_x > 0.0 {
                    self.x = platform.x
                        + platform.width
                        + crate::constants::ENEMY_COLLISION_PLATFORM_OFFSET;
                } else {
                    self.x =
                        platform.x - self.width - crate::constants::ENEMY_COLLISION_PLATFORM_OFFSET;
                }
            }
        }
    }

    #[inline]
    pub fn check_edge(&mut self, platforms: &[crate::platform::Platform]) {
        if !self.alive || !self.on_ground {
            return;
        }

        let check_offset_x = if self.vel_x > 0.0 {
            self.width + crate::constants::ENEMY_EDGE_CHECK_OFFSET
        } else {
            -crate::constants::ENEMY_EDGE_CHECK_OFFSET
        };

        let check_x = self.x + check_offset_x;
        let check_y = self.y + self.height + crate::constants::ENEMY_EDGE_CHECK_Y_OFFSET;

        let mut found_platform_below = false;

        for platform in platforms {
            if check_x >= platform.x
                && check_x <= platform.x + platform.width
                && check_y >= platform.y
                && check_y <= platform.y + platform.height
            {
                found_platform_below = true;
                break;
            }
        }

        if !found_platform_below {
            self.vel_x = -self.vel_x;
            if self.vel_x > 0.0 {
                self.x += crate::constants::ENEMY_EDGE_CHECK_OFFSET;
            } else {
                self.x -= crate::constants::ENEMY_EDGE_CHECK_OFFSET;
            }
        }
    }

    pub fn check_ground_collision(&mut self, ground_y: f32) {
        if !self.alive {
            return;
        }

        if self.y + self.height >= ground_y {
            self.y = ground_y - self.height;
            self.vel_y = 0.0;
            self.on_ground = true;
        }
    }

    pub fn check_player_collision(
        &mut self,
        player_x: f32,
        player_y: f32,
        player_w: f32,
        player_h: f32,
        player_vel_y: f32,
    ) -> Option<bool> {
        if !self.alive {
            return None;
        }

        if self.check_collision(player_x, player_y, player_w, player_h) {
            let player_bottom = player_y + player_h;
            let enemy_top = self.y;
            let enemy_center_y = self.y + self.height / 2.0;

            let is_on_top = player_vel_y >= crate::constants::PLAYER_COLLISION_VELOCITY_THRESHOLD
                && player_bottom
                    <= enemy_center_y + crate::constants::PLAYER_ENEMY_TOP_COLLISION_THRESHOLD;

            let is_falling_on_top = player_vel_y > 0.0
                && player_bottom
                    <= enemy_top + self.height * crate::constants::PLAYER_ENEMY_FALLING_THRESHOLD;

            if is_on_top || is_falling_on_top {
                self.alive = false;
                return Some(false);
            }

            return Some(true);
        }
        None
    }

    fn check_collision(&self, other_x: f32, other_y: f32, other_w: f32, other_h: f32) -> bool {
        other_x < self.x + self.width
            && other_x + other_w > self.x
            && other_y < self.y + self.height
            && other_y + other_h > self.y
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32) {
        if !self.alive {
            return;
        }

        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        if let Some(tex) = &self.texture {
            let frame_w = tex.width() / 4.0;
            let frame_h = tex.height();
            let source_x = self.anim_frame as f32 * frame_w;
            let source = Rect::new(source_x, 0.0, frame_w, frame_h);
            let params = DrawTextureParams {
                dest_size: Some(vec2(self.width, self.height)),
                source: Some(source),
                rotation: 0.0,
                flip_x: !self.facing_right,
                flip_y: false,
                pivot: None,
            };
            draw_texture_ex(&**tex, screen_x, screen_y, WHITE, params);
        } else {
            draw_rectangle(screen_x, screen_y, self.width, self.height, GRAY);
            draw_rectangle_lines(screen_x, screen_y, self.width, self.height, 2.0, BLACK);
            draw_circle(screen_x + 6.0, screen_y + 8.0, 2.0, BLACK);
            draw_circle(screen_x + 18.0, screen_y + 8.0, 2.0, BLACK);
        }
    }
}

pub fn create_level_enemies(level: usize, textures: Option<&[Rc<Texture2D>]>) -> Vec<Enemy> {
    let mut enemies = Vec::with_capacity(crate::constants::ESTIMATED_ENEMIES_PER_LEVEL);

    let texture_for = |idx: usize| -> Option<Rc<Texture2D>> {
        textures.and_then(|t| t.get(idx % t.len()).cloned())
    };

    match level {
        1 => {
            enemies.push(Enemy::new(300.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(500.0, 370.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(700.0, 320.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(900.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1200.0, 370.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1400.0, 320.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1600.0, 370.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2100.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2300.0, 370.0, texture_for(enemies.len())));
        }
        2 => {
            enemies.push(Enemy::new(200.0, 470.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(350.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(500.0, 370.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(650.0, 320.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(800.0, 270.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(950.0, 220.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1100.0, 270.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1250.0, 320.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1400.0, 370.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1550.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1700.0, 470.0, texture_for(enemies.len())));
        }
        3 => {
            enemies.push(Enemy::new(200.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(500.0, 470.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(700.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(900.0, 370.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1150.0, 320.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1350.0, 270.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1650.0, 320.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1900.0, 370.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2100.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2350.0, 470.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2550.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(400.0, 270.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(600.0, 220.0, texture_for(enemies.len())));
        }
        4 => {
            enemies.push(Enemy::new(150.0, 470.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(300.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(450.0, 470.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(600.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(750.0, 470.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(900.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1100.0, 320.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1300.0, 270.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1500.0, 220.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1700.0, 170.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1900.0, 370.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2100.0, 320.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2300.0, 270.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2500.0, 220.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2700.0, 320.0, texture_for(enemies.len())));
        }
        5 => {
            enemies.push(Enemy::new(350.0, 480.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(620.0, 430.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(880.0, 370.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1150.0, 320.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1420.0, 400.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1680.0, 250.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(1880.0, 200.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2120.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2380.0, 360.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2580.0, 300.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(2780.0, 260.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(3020.0, 480.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(3220.0, 420.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(3420.0, 360.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(3620.0, 320.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(3820.0, 260.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(4020.0, 220.0, texture_for(enemies.len())));
            enemies.push(Enemy::new(4100.0, 500.0, texture_for(enemies.len())));
        }
        _ => {}
    }

    enemies
}
