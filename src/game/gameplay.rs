use crate::constants::*;
use crate::systems::Particle;
use macroquad::prelude::*;

use super::{Game, GameState, ControlAction};

impl Game {
    pub fn update_playing(&mut self, dt: f32) {
        let effective_dt = if self.assist_mode {
            dt * ASSIST_MODE_SLOW_MOTION
        } else {
            dt
        };
        if is_key_pressed(KeyCode::P) {
            self.state = GameState::Pause;
            self.pause_selection = 0;
            self.came_from_pause = false;
            return;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.transition_to_menu();
            self.menu_selection = 0;
            return;
        }
        if self.level_start_fade_timer > 0.0 {
            self.level_start_fade_timer -= dt;
        }
        self.time_remaining -= effective_dt;
        if self.time_remaining <= 0.0 {
            self.time_remaining = 0.0;
            self.audio.play_death();
            self.state = GameState::GameOver;
            return;
        }
        let p1_left = self.is_control_pressed(&self.player1_controls, ControlAction::Left);
        let p1_right = self.is_control_pressed(&self.player1_controls, ControlAction::Right);
        self.player.handle_movement_custom(p1_left, p1_right);
        self.player.update(effective_dt);
        let (px, py, pw, ph) = self.player.get_rect();
        for checkpoint in &mut self.checkpoints {
            if checkpoint.check_activation(px, py, pw, ph) {
                self.last_checkpoint_pos = Some((checkpoint.x, checkpoint.y));
                self.score += SCORE_CHECKPOINT;
                self.audio.play_coin();
            }
        }
        Self::check_player_platform_collisions(
            &mut self.player,
            &self.platforms,
            (px, py, pw, ph),
        );
        let p1_jump = self.is_control_pressed(&self.player1_controls, ControlAction::Jump);
        let jumped = self.player.handle_jump_custom(p1_jump);
        if jumped {
            self.audio.play_jump(self.is_easter_egg());
            self.camera_shake.trigger_jump();
        }
        self.player.update_animation(dt);
        if self.player.on_ground && self.player.vel_x.abs() > MIN_VELOCITY_FOR_FOOTSTEP {
            self.footstep_timer += effective_dt;
            if self.footstep_timer >= FOOTSTEP_INTERVAL {
                self.audio.play_footstep(self.is_easter_egg());
                self.footstep_timer = 0.0;
                self.camera_shake.trigger_footstep();
            }
        } else {
            self.footstep_timer = 0.0;
        }
        let player_left = self.player.x;
        let player_right = self.player.x + self.player.width;
        if player_left < 0.0 {
            self.player.x = 0.0;
            self.player.vel_x = 0.0;
        }
        if player_right > WORLD_WIDTH {
            self.player.x = WORLD_WIDTH - self.player.width;
            self.player.vel_x = 0.0;
        }
        if self.player.y > FALL_DEATH_Y {
            self.handle_player_death();
        }
        let mut player_vel_y_update = None;
        for enemy in &mut self.enemies {
            if !enemy.alive {
                continue;
            }
            enemy.update(effective_dt);
            Self::check_enemy_platform_collisions(enemy, &self.platforms);
            if enemy.on_ground {
                enemy.check_edge(&self.platforms);
            }
            enemy.check_ground_collision(GROUND_Y);
            match enemy.check_player_collision(px, py, pw, ph, self.player.vel_y) {
                Some(true) => {
                    self.handle_player_death();
                    break;
                }
                Some(false) => {
                    let enemy_x = enemy.x;
                    let enemy_y = enemy.y;
                    let enemy_width = enemy.width;
                    let enemy_height = enemy.height;
                    player_vel_y_update = Some(JUMP_FORCE * JUMP_BOUNCE_MULTIPLIER);
                    self.audio.play_enemy_death();
                    self.score += SCORE_ENEMY;
                    self.camera_shake.trigger_kill();
                    for _ in 0..PARTICLE_COUNT {
                        let angle = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
                        let speed = rand::gen_range(40.0, 100.0);
                        self.particles.push(Particle::new(
                            enemy_x + enemy_width / 2.0,
                            enemy_y + enemy_height / 2.0,
                            angle.cos() * speed,
                            angle.sin() * speed,
                        ));
                    }
                }
                None => {}
            }
        }
        if let Some(vel_y) = player_vel_y_update {
            self.player.vel_y = vel_y;
        }
        let mut coins_to_collect = Vec::new();
        for coin in &mut self.coins {
            if coin.collected {
                continue;
            }
            coin.update(effective_dt);
            if coin.check_collection(px, py, pw, ph) {
                coins_to_collect.push((coin.x, coin.y));
            }
        }
        for (coin_x, coin_y) in coins_to_collect {
            self.handle_coin_collection(coin_x, coin_y);
        }
        if self.player.x > LEVEL_COMPLETE_X || self.coins_collected >= self.total_coins {
            let time_bonus = (self.time_remaining * SCORE_TIME_BONUS) as u32;
            self.score += SCORE_LEVEL_COMPLETE + time_bonus;
            self.audio.play_level_complete();
            if self.current_level < MAX_LEVELS
                && self.current_level < self.unlocked_levels.len()
            {
                self.unlocked_levels[self.current_level] = true;
            }
            if let Err(e) = self.save_game(0) {
                let error_msg = format!("Error saving game: {}", e);
                eprintln!("{}", error_msg);
                self.show_error(error_msg);
            }
            self.state = GameState::LevelComplete;
        }
        let screen_width = screen_width();
        self.camera.update(self.player.x, screen_width, self.camera_shake.get_offset());
    }
}

