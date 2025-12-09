use crate::coin::create_level_coins;
use crate::constants::*;
use crate::enemy::Enemy;
use crate::platform::Platform;
use crate::player::Player;
use crate::save::SaveData;
use crate::systems::{CoinBounce, Particle};
use macroquad::prelude::*;

use super::{ControlAction, Game, GameState, PlayerControls};

impl Game {
    pub fn get_common_resolutions() -> Vec<(u32, u32)> {
        vec![
            (800, 600),
            (1024, 768),
            (1280, 720),
            (1280, 1024),
            (1366, 768),
            (1600, 900),
            (1920, 1080),
            (2560, 1440),
            (3840, 2160),
        ]
    }

    pub fn init_level_info_cache() -> Vec<(String, usize, Color)> {
        let mut cache = Vec::with_capacity(MAX_LEVELS);
        for level in 1..=MAX_LEVELS {
            let coins = create_level_coins(level);
            let coin_count = coins.len();
            let (difficulty, color) = match level {
                1 => ("EASY".to_string(), GREEN),
                2 => ("MEDIUM".to_string(), YELLOW),
                3 => ("HARD".to_string(), Color::new(1.0, 0.65, 0.0, 1.0)),
                4 => ("EXPERT".to_string(), RED),
                5 => ("INSANE".to_string(), BLACK),
                _ => ("UNKNOWN".to_string(), GRAY),
            };
            cache.push((difficulty, coin_count, color));
        }
        cache
    }

    pub fn apply_resolution(&self) {
        if self.resolution_index < self.available_resolutions.len() {
            let (width, height) = self.available_resolutions[self.resolution_index];
            request_new_screen_size(width as f32, height as f32);
        }
    }

    pub fn save_game(&self, slot: usize) -> Result<(), String> {
        let time_taken = TIME_LIMIT - self.time_remaining;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let save_data = SaveData {
            current_level: self.current_level,
            unlocked_levels: self.unlocked_levels.clone(),
            lives: self.lives,
            score: self.score,
            coins_collected: self.coins_collected,
            total_coins: self.total_coins,
            time_remaining: self.time_remaining,
            time_taken,
            timestamp,
            last_checkpoint_pos: self.last_checkpoint_pos,
            player_name: self.player_name.clone(),
            tutorial_completed: self.tutorial_completed,
            versus_played: self.versus_played,
        };
        let path = SaveData::get_save_path(slot)?;
        save_data.save_to_file(&path)
    }

    pub fn load_game(&mut self, slot: usize) -> Result<(), String> {
        let save_data = SaveData::load_slot(slot)?;
        self.current_level = save_data.current_level;
        self.unlocked_levels = save_data.unlocked_levels;
        self.lives = save_data.lives;
        self.score = save_data.score;
        let default_total = create_level_coins(self.current_level).len() as u32;
        self.total_coins = if save_data.total_coins == 0 {
            default_total
        } else {
            save_data.total_coins
        };
        self.coins_collected = save_data.coins_collected.min(self.total_coins);
        self.time_remaining = if save_data.time_remaining > 0.0 {
            save_data.time_remaining
        } else {
            TIME_LIMIT
        };
        self.last_checkpoint_pos = save_data.last_checkpoint_pos;
        self.player_name = save_data.player_name;
        self.tutorial_completed = save_data.tutorial_completed;
        self.versus_played = save_data.versus_played;
        Ok(())
    }

    pub fn is_easter_egg(&self) -> bool {
        self.player_name.to_lowercase() == "guicybercode"
    }

    pub fn show_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = 5.0;
    }

    pub fn handle_player_death(&mut self) {
        self.audio.play_death();
        if self.lives > 0 {
            self.lives -= 1;
        }
        if self.lives == 0 {
            self.game_over_fade_timer = GAME_OVER_FADE_TIMER;
            self.state = GameState::GameOver;
        } else {
            self.respawn_timer = RESPAWN_TIMER;
            self.state = GameState::Respawn;
        }
    }

    pub fn handle_coin_collection(&mut self, coin_x: f32, coin_y: f32) {
        self.coins_collected += 1;
        self.score += SCORE_COIN;
        self.audio.play_coin();
        self.coin_bounces.push(CoinBounce::new(
            coin_x + COIN_SIZE / 2.0,
            coin_y + COIN_SIZE / 2.0,
        ));
        for _ in 0..PARTICLE_COUNT {
            let angle = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
            let speed = rand::gen_range(40.0, 90.0);
            self.particles.push(Particle::new(
                coin_x + COIN_SIZE / 2.0,
                coin_y + COIN_SIZE / 2.0,
                angle.cos() * speed,
                angle.sin() * speed,
            ));
        }
    }

    pub fn handle_player_death_versus_p1(&mut self) {
        self.audio.play_death();
        if self.lives > 0 {
            self.lives -= 1;
        }
        if self.lives == 0 {
            self.game_over_fade_timer = GAME_OVER_FADE_TIMER;
            self.state = GameState::GameOver;
        } else {
            self.respawn_timer_p1 = RESPAWN_TIMER;
        }
    }

    pub fn handle_player_death_versus_p2(&mut self) {
        self.audio.play_death();
        if self.lives > 0 {
            self.lives -= 1;
        }
        if self.lives == 0 {
            self.game_over_fade_timer = GAME_OVER_FADE_TIMER;
            self.state = GameState::GameOver;
        } else {
            self.respawn_timer_p2 = RESPAWN_TIMER;
        }
    }

    pub fn start_transition(&mut self, target_state: GameState) {
        if matches!(self.state, GameState::Menu) {
            self.state = target_state;
        } else {
            self.transition.start(target_state);
        }
    }

    pub fn transition_to_menu(&mut self) {
        if !self.splash_shown {
            self.splash_timer = 0.0;
            self.start_transition(GameState::Splash);
        } else {
            self.start_transition(GameState::Menu);
        }
    }

    pub fn is_control_pressed(&self, controls: &PlayerControls, action: ControlAction) -> bool {
        match action {
            ControlAction::Left => controls.left.map(|k| is_key_down(k)).unwrap_or(false),
            ControlAction::Right => controls.right.map(|k| is_key_down(k)).unwrap_or(false),
            ControlAction::Jump => controls.jump.map(|k| is_key_down(k)).unwrap_or(false),
        }
    }

    pub fn update_transition(&mut self, dt: f32) {
        if let Some(target) = self.transition.update(dt) {
            self.state = target;
        }
    }

    pub fn draw_transition(&self) {
        self.transition.draw();
    }

    pub fn draw_level_start_fade(&self) {
        if self.level_start_fade_timer > 0.0 {
            let fade_progress = self.level_start_fade_timer / LEVEL_START_FADE_TIMER;
            let eased_progress = fade_progress * fade_progress;
            let fade_alpha = eased_progress.min(1.0);
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, fade_alpha),
            );
        }
    }

    pub fn check_for_new_saves(&mut self) {
        let saves = SaveData::list_all_saves();
        let mut newest_timestamp = self.last_save_timestamp;
        for (_, save_opt) in saves {
            if let Some(save) = save_opt {
                if save.timestamp > newest_timestamp {
                    newest_timestamp = save.timestamp;
                }
            }
        }
        if newest_timestamp > self.last_save_timestamp {
            self.has_new_save = true;
            self.last_save_timestamp = newest_timestamp;
        }
    }

    pub fn get_level_info(&self, level: usize) -> (String, usize, Color) {
        if level > 0 && level <= self.level_info_cache.len() {
            self.level_info_cache[level - 1].clone()
        } else {
            ("UNKNOWN".to_string(), 0, GRAY)
        }
    }

    #[inline]
    pub fn is_nearby_for_collision(
        x1: f32,
        y1: f32,
        w1: f32,
        h1: f32,
        x2: f32,
        y2: f32,
        w2: f32,
        h2: f32,
        margin: f32,
    ) -> bool {
        x2 + w2 >= x1 - margin
            && x2 <= x1 + w1 + margin
            && y2 + h2 >= y1 - margin
            && y2 <= y1 + h1 + margin
    }

    pub fn check_player_platform_collisions(
        player: &mut Player,
        platforms: &[Platform],
        player_rect: (f32, f32, f32, f32),
    ) {
        let (px, py, pw, ph) = player_rect;
        for platform in platforms {
            if Self::is_nearby_for_collision(
                px,
                py,
                pw,
                ph,
                platform.x,
                platform.y,
                platform.width,
                platform.height,
                COLLISION_MARGIN,
            ) {
                player.check_platform_collision(platform);
            }
        }
    }

    pub fn check_enemy_platform_collisions(enemy: &mut Enemy, platforms: &[Platform]) {
        for platform in platforms {
            if Self::is_nearby_for_collision(
                enemy.x,
                enemy.y,
                enemy.width,
                enemy.height,
                platform.x,
                platform.y,
                platform.width,
                platform.height,
                COLLISION_MARGIN,
            ) {
                enemy.check_platform_collision(platform);
            }
        }
    }

    pub fn calculate_versus_points(streak: u32) -> u32 {
        if streak == 0 {
            return 200;
        }
        let exp = streak.saturating_sub(1).min(10);
        let multiplier = 1u32.checked_shl(exp).unwrap_or(u32::MAX);
        200u32.saturating_mul(multiplier)
    }

    pub fn is_player_on_platform(
        player_x: f32,
        player_y: f32,
        player_w: f32,
        platforms: &[Platform],
    ) -> Option<f32> {
        let player_center_x = player_x + player_w / 2.0;
        for platform in platforms {
            if player_center_x >= platform.x
                && player_center_x <= platform.x + platform.width
                && (player_y + PLAYER_HEIGHT - platform.y).abs() < 5.0
            {
                return Some(platform.y);
            }
        }
        None
    }

    pub fn ensure_player_grounded(player: &mut Player, platforms: &[Platform]) {
        if player.on_ground && player.vel_y == 0.0 {
            let (px, py, pw, _ph) = player.get_rect();
            if let Some(platform_y) = Self::is_player_on_platform(px, py, pw, platforms) {
                player.y = platform_y - PLAYER_HEIGHT;
            } else {
                player.y = GROUND_Y - PLAYER_HEIGHT;
            }
        }
    }

    pub fn update_versus_player_physics(
        player: &mut Player,
        left: bool,
        right: bool,
        jump: bool,
        platforms: &[Platform],
        dt: f32,
    ) -> bool {
        player.handle_movement_custom(left, right);
        player.update(dt);
        let (px, py, pw, ph) = player.get_rect();
        Self::check_player_platform_collisions(player, platforms, (px, py, pw, ph));
        Self::ensure_player_grounded(player, platforms);
        player.update_animation(dt);
        player.handle_jump_custom(jump)
    }

    pub fn load_level(
        &mut self,
        level: usize,
        use_checkpoint: bool,
        restored_time: Option<f32>,
        restored_coins: Option<u32>,
    ) {
        use crate::camera::Camera;
        use crate::checkpoint::create_level_checkpoints;
        use crate::enemy::create_level_enemies;
        use crate::platform::create_level_platforms;

        let platforms = create_level_platforms(level);
        let coins = create_level_coins(level);
        let total_coins = coins.len() as u32;
        let textures = if self.enemy_textures.is_empty() {
            None
        } else {
            Some(self.enemy_textures.as_slice())
        };
        let mut enemies = create_level_enemies(level, textures);
        let mut checkpoints = create_level_checkpoints(level);
        if use_checkpoint && self.last_checkpoint_pos.is_some() {
            if let Some((checkpoint_x, _)) = self.last_checkpoint_pos {
                for checkpoint in &mut checkpoints {
                    if checkpoint.x <= checkpoint_x {
                        checkpoint.activated = true;
                    }
                }
            }
        }
        for enemy in &mut enemies {
            let enemy_center_x = enemy.x + enemy.width / 2.0;
            let mut best_platform_y = GROUND_Y;
            let mut found_platform = false;
            for platform in &platforms {
                if enemy_center_x >= platform.x
                    && enemy_center_x <= platform.x + platform.width
                    && platform.y <= best_platform_y
                {
                    best_platform_y = platform.y;
                    found_platform = true;
                }
            }
            enemy.y = best_platform_y - enemy.height;
            enemy.on_ground = true;
            if !found_platform {
                enemy.y = GROUND_Y - enemy.height;
            }
        }
        let (player_start_x, player_start_y) =
            if let (true, Some((checkpoint_x, _))) = (use_checkpoint, self.last_checkpoint_pos) {
                (checkpoint_x + 50.0, GROUND_Y - PLAYER_HEIGHT)
            } else {
                (50.0, GROUND_Y - PLAYER_HEIGHT)
            };
        let mut player = Player::new(
            player_start_x,
            player_start_y,
            self.player_sprite_texture_p1
                .as_ref()
                .map(|t| std::rc::Rc::clone(t)),
            self.player_sprite_texture_p2
                .as_ref()
                .map(|t| std::rc::Rc::clone(t)),
        );
        player.on_ground = true;
        player.vel_y = 0.0;
        self.player = player;
        self.enemies = enemies;
        self.platforms = platforms;
        self.coins = coins;
        self.checkpoints = checkpoints;
        self.camera = Camera::new();
        self.coins_collected = restored_coins.unwrap_or(0).min(total_coins);
        self.total_coins = total_coins;
        self.current_level = level;
        self.time_remaining = restored_time.unwrap_or(TIME_LIMIT);
        self.footstep_timer = 0.0;
        if self.coins_collected > 0 {
            let mut remaining = self.coins_collected;
            for coin in &mut self.coins {
                if remaining == 0 {
                    break;
                }
                coin.collected = true;
                remaining -= 1;
            }
        }
        self.level_start_fade_timer = LEVEL_START_FADE_TIMER;
    }

    pub fn load_versus_map(&mut self) {
        use crate::camera::Camera;

        let mut platforms = Vec::with_capacity(ESTIMATED_PLATFORMS_PER_LEVEL);
        let screen_w = SCREEN_WIDTH as f32;
        platforms.push(Platform::new(0.0, GROUND_Y, screen_w, 50.0));
        platforms.push(Platform::new(100.0, 450.0, 150.0, 20.0));
        platforms.push(Platform::new(screen_w - 250.0, 450.0, 150.0, 20.0));
        platforms.push(Platform::new(200.0, 350.0, 120.0, 20.0));
        platforms.push(Platform::new(screen_w - 320.0, 350.0, 120.0, 20.0));
        platforms.push(Platform::new(150.0, 250.0, 100.0, 20.0));
        platforms.push(Platform::new(screen_w - 250.0, 250.0, 100.0, 20.0));
        platforms.push(Platform::new(screen_w / 2.0 - 50.0, 400.0, 100.0, 20.0));
        platforms.push(Platform::new(50.0, 500.0, 40.0, 50.0));
        platforms.push(Platform::new(screen_w - 90.0, 500.0, 40.0, 50.0));
        self.versus_platforms = platforms;
        self.player = Player::new(
            100.0,
            GROUND_Y - PLAYER_HEIGHT,
            self.player_sprite_texture_p1
                .as_ref()
                .map(|t| std::rc::Rc::clone(t)),
            self.player_sprite_texture_p2
                .as_ref()
                .map(|t| std::rc::Rc::clone(t)),
        );
        self.player.on_ground = true;
        self.player.vel_y = 0.0;
        self.player.y = GROUND_Y - PLAYER_HEIGHT;
        self.player2 = Some(Player::new(
            screen_w - 100.0 - PLAYER_WIDTH,
            GROUND_Y - PLAYER_HEIGHT,
            self.player_sprite_texture_p1
                .as_ref()
                .map(|t| std::rc::Rc::clone(t)),
            self.player_sprite_texture_p2
                .as_ref()
                .map(|t| std::rc::Rc::clone(t)),
        ));
        if let Some(ref mut p2) = self.player2 {
            p2.on_ground = true;
            p2.vel_y = 0.0;
            p2.y = GROUND_Y - PLAYER_HEIGHT;
        }
        self.player1_score = 0;
        self.player2_score = 0;
        self.player1_streak = 0;
        self.player2_streak = 0;
        self.player1_points = 0;
        self.player2_points = 0;
        self.respawn_timer_p1 = 0.0;
        self.respawn_timer_p2 = 0.0;
        self.versus_time_remaining = 600.0;
        self.camera = Camera::new();
    }
}
