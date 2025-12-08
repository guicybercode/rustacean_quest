use crate::audio::AudioManager;
use crate::camera::Camera;
use crate::checkpoint::{create_level_checkpoints, Checkpoint};
use crate::coin::{create_level_coins, Coin};
use crate::constants::*;
use crate::enemy::{create_level_enemies, Enemy};
use crate::name_filter;
use crate::platform::{create_level_platforms, Platform};
use crate::player::Player;
use crate::save::SaveData;
use macroquad::prelude::*;

pub enum GameState {
    Splash,
    Menu,
    MenuExitConfirm,
    Credits,
    Settings,
    Controls,
    LevelSelect,
    Playing,
    GameOver,
    LevelComplete,
    Versus,
    VersusEnd,
    Coop,
    Respawn,
    ContinueMenu,
    NameInput,
    Tutorial,
    Pause,
}
#[derive(Clone, Copy, PartialEq)]
pub enum ControlAction {
    Left,
    Right,
    Jump,
}

#[derive(Clone)]
pub struct PlayerControls {
    pub left: Option<KeyCode>,
    pub right: Option<KeyCode>,
    pub jump: Option<KeyCode>,
    pub left_gamepad: Option<u8>,
    pub right_gamepad: Option<u8>,
    pub jump_gamepad: Option<u8>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ContinueMode {
    View,
    DeleteConfirm,
}
pub struct Game {
    player: Player,
    enemies: Vec<Enemy>,
    platforms: Vec<Platform>,
    coins: Vec<Coin>,
    checkpoints: Vec<Checkpoint>,
    camera: Camera,
    audio: AudioManager,
    state: GameState,
    coins_collected: u32,
    total_coins: u32,
    menu_selection: usize,
    menu_animation_time: f32,
    level_selection: usize,
    current_level: usize,
    unlocked_levels: Vec<bool>,
    last_checkpoint_pos: Option<(f32, f32)>,
    time_remaining: f32,
    settings_selection: usize,
    sound_enabled: bool,
    resolution_index: usize,
    available_resolutions: Vec<(u32, u32)>,
    #[allow(dead_code)]
    fullscreen: bool,
    score: u32,
    lives: u32,
    respawn_timer: f32,
    game_over_fade_timer: f32,
    level_start_fade_timer: f32,
    footstep_timer: f32,
    player2: Option<Player>,
    player1_score: u32,
    player2_score: u32,
    player1_streak: u32,
    player2_streak: u32,
    player1_points: u32,
    player2_points: u32,
    versus_platforms: Vec<Platform>,
    respawn_timer_p1: f32,
    respawn_timer_p2: f32,
    versus_time_remaining: f32,
    player_name: String,
    continue_selection: usize,
    continue_mode: ContinueMode,
    name_input: String,
    name_input_error: Option<String>,
    tutorial_page: usize,
    tutorial_completed: bool,
    transition_timer: f32,
    transition_alpha: f32,
    transition_target_state: Option<GameState>,
    versus_played: bool,
    has_new_save: bool,
    last_save_timestamp: u64,
    save_check_timer: f32,
    level_info_cache: Vec<(String, usize, Color)>,
    pause_selection: usize,
    came_from_pause: bool,
    player_sprite_texture_p1: Option<std::rc::Rc<Texture2D>>,
    player_sprite_texture_p2: Option<std::rc::Rc<Texture2D>>,
    enemy_textures: Vec<std::rc::Rc<Texture2D>>,
    splash_timer: f32,
    previous_menu_selection: usize,
    splash_shown: bool,
    player1_controls: PlayerControls,
    player2_controls: PlayerControls,
    controls_selection: usize,
    controls_player: usize,
    controls_waiting_input: Option<(usize, ControlAction)>,
    error_message: Option<String>,
    error_timer: f32,
    #[allow(dead_code)]
    asset_load_errors: Vec<String>,
    difficulty_multiplier: f32,
    colorblind_mode: bool,
    font_size_scale: f32,
    assist_mode: bool,
    camera_shake_timer: f32,
    camera_shake_intensity: f32,
    particles: Vec<(f32, f32, f32, f32, f32)>,
}
impl Game {
    pub async fn new() -> Self {
        let mut unlocked_levels = vec![false; MAX_LEVELS];
        unlocked_levels[0] = true;
        let mut audio = AudioManager::new();
        audio.load_sounds().await;
        let mut asset_load_errors = Vec::new();
        let mut enemy_textures = Vec::with_capacity(2);
        let player_sprite_texture_p1 = match load_texture("assets/crab1.png").await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest);
                let rc = std::rc::Rc::new(texture);
                enemy_textures.push(std::rc::Rc::clone(&rc));
                Some(rc)
            }
            Err(_) => match load_texture("assets/rustcean_p1.png").await {
                Ok(texture) => {
                    texture.set_filter(FilterMode::Nearest);
                    Some(std::rc::Rc::new(texture))
                }
                Err(e) => {
                    let error_msg = format!("Erro ao carregar sprite P1: {:?}", e);
                    eprintln!("{}", error_msg);
                    asset_load_errors.push(error_msg);
                    None
                }
            },
        };
        let player_sprite_texture_p2 = match load_texture("assets/crab2.png").await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest);
                let rc = std::rc::Rc::new(texture);
                enemy_textures.push(std::rc::Rc::clone(&rc));
                Some(rc)
            }
            Err(_) => match load_texture("assets/rustcean_p2.png").await {
                Ok(texture) => {
                    texture.set_filter(FilterMode::Nearest);
                    Some(std::rc::Rc::new(texture))
                }
                Err(e) => {
                    let error_msg = format!("Erro ao carregar sprite P2: {:?}", e);
                    eprintln!("{}", error_msg);
                    asset_load_errors.push(error_msg);
                    None
                }
            },
        };
        Self {
            player: Player::new(
                50.0,
                400.0,
                player_sprite_texture_p1
                    .as_ref()
                    .map(|t| std::rc::Rc::clone(t)),
                player_sprite_texture_p2
                    .as_ref()
                    .map(|t| std::rc::Rc::clone(t)),
            ),
            enemies: Vec::with_capacity(ESTIMATED_ENEMIES_PER_LEVEL),
            platforms: Vec::with_capacity(ESTIMATED_PLATFORMS_PER_LEVEL),
            coins: Vec::with_capacity(ESTIMATED_COINS_PER_LEVEL),
            checkpoints: Vec::with_capacity(ESTIMATED_CHECKPOINTS_PER_LEVEL),
            camera: Camera::new(),
            audio,
            state: GameState::Splash,
            coins_collected: 0,
            total_coins: 0,
            menu_selection: 0,
            menu_animation_time: 0.0,
            level_selection: 0,
            current_level: 1,
            unlocked_levels,
            last_checkpoint_pos: None,
            time_remaining: 0.0,
            settings_selection: 0,
            sound_enabled: true,
            resolution_index: 0,
            available_resolutions: Self::get_common_resolutions(),
            fullscreen: false,
            score: 0,
            lives: DEFAULT_LIVES,
            respawn_timer: 0.0,
            game_over_fade_timer: 0.0,
            level_start_fade_timer: LEVEL_START_FADE_TIMER,
            footstep_timer: 0.0,
            player2: None,
            player1_score: 0,
            player2_score: 0,
            player1_streak: 0,
            player2_streak: 0,
            player1_points: 0,
            player2_points: 0,
            versus_platforms: Vec::with_capacity(10),
            respawn_timer_p1: 0.0,
            respawn_timer_p2: 0.0,
            versus_time_remaining: 600.0,
            player_name: String::new(),
            continue_selection: 0,
            continue_mode: ContinueMode::View,
            name_input: String::new(),
            name_input_error: None,
            tutorial_page: 0,
            tutorial_completed: false,
            transition_timer: 0.0,
            transition_alpha: 0.0,
            transition_target_state: None,
            versus_played: false,
            has_new_save: false,
            last_save_timestamp: 0,
            save_check_timer: 0.0,
            level_info_cache: Self::init_level_info_cache(),
            pause_selection: 0,
            came_from_pause: false,
            player_sprite_texture_p1,
            player_sprite_texture_p2,
            enemy_textures,
            splash_timer: 0.0,
            previous_menu_selection: 0,
            splash_shown: false,
            player1_controls: PlayerControls {
                left: Some(KeyCode::A),
                right: Some(KeyCode::D),
                jump: Some(KeyCode::W),
                left_gamepad: None,
                right_gamepad: None,
                jump_gamepad: None,
            },
            player2_controls: PlayerControls {
                left: Some(KeyCode::Left),
                right: Some(KeyCode::Right),
                jump: Some(KeyCode::Up),
                left_gamepad: None,
                right_gamepad: None,
                jump_gamepad: None,
            },
            controls_selection: 0,
            controls_player: 1,
            controls_waiting_input: None,
            error_message: None,
            error_timer: 0.0,
            asset_load_errors: Vec::new(),
            difficulty_multiplier: DIFFICULTY_NORMAL,
            colorblind_mode: false,
            font_size_scale: 1.0,
            assist_mode: false,
            camera_shake_timer: 0.0,
            camera_shake_intensity: 0.0,
            particles: Vec::with_capacity(PARTICLE_COUNT * 10),
        }
    }
    fn get_common_resolutions() -> Vec<(u32, u32)> {
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
    fn init_level_info_cache() -> Vec<(String, usize, Color)> {
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
    fn apply_resolution(&self) {
        if self.resolution_index < self.available_resolutions.len() {
            let (width, height) = self.available_resolutions[self.resolution_index];
            request_new_screen_size(width as f32, height as f32);
        }
    }
    fn save_game(&self, slot: usize) -> Result<(), String> {
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
        let path = SaveData::get_save_path(slot);
        save_data.save_to_file(&path)
    }
    fn load_game(&mut self, slot: usize) -> Result<(), String> {
        let path = SaveData::get_save_path(slot);
        let save_data = SaveData::load_from_file(&path)?;
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
    #[allow(dead_code)]
    fn has_save_file(&self) -> bool {
        for slot in 0..3 {
            let path = SaveData::get_save_path(slot);
            if SaveData::save_exists(&path) {
                return true;
            }
        }
        false
    }
    fn is_easter_egg(&self) -> bool {
        self.player_name.to_lowercase() == "guicybercode"
    }
    fn show_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = 5.0;
    }
    fn start_transition(&mut self, target_state: GameState) {
        if matches!(self.state, GameState::Menu) {
            self.state = target_state;
            self.transition_timer = 0.0;
            self.transition_alpha = 0.0;
            self.transition_target_state = None;
        } else {
            self.transition_timer = 0.0;
            self.transition_alpha = 1.0;
            self.transition_target_state = Some(target_state);
        }
    }
    fn transition_to_menu(&mut self) {
        if !self.splash_shown {
            self.splash_timer = 0.0;
            self.start_transition(GameState::Splash);
        } else {
            self.start_transition(GameState::Menu);
        }
    }
    fn is_control_pressed(&self, controls: &PlayerControls, action: ControlAction) -> bool {
        match action {
            ControlAction::Left => controls.left.map(|k| is_key_down(k)).unwrap_or(false),
            ControlAction::Right => controls.right.map(|k| is_key_down(k)).unwrap_or(false),
            ControlAction::Jump => controls.jump.map(|k| is_key_down(k)).unwrap_or(false),
        }
    }
    fn update_transition(&mut self, dt: f32) {
        if self.transition_target_state.is_none() {
            if self.transition_timer > 0.0 {
                self.transition_timer = 0.0;
                self.transition_alpha = 0.0;
            }
            return;
        }
        self.transition_timer += dt;
        let progress = (self.transition_timer / TRANSITION_DURATION).min(1.0);
        self.transition_alpha = 1.0 - progress;
        if progress >= 1.0 {
            if let Some(target) = self.transition_target_state.take() {
                self.state = target;
            }
            self.transition_timer = 0.0;
            self.transition_alpha = 0.0;
        }
    }
    fn draw_transition(&self) {
        if self.transition_timer > 0.0 && self.transition_alpha > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, self.transition_alpha),
            );
        }
    }
    fn check_for_new_saves(&mut self) {
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
    fn get_level_info(&self, level: usize) -> (String, usize, Color) {
        if level > 0 && level <= self.level_info_cache.len() {
            self.level_info_cache[level - 1].clone()
        } else {
            ("UNKNOWN".to_string(), 0, GRAY)
        }
    }
    fn load_level(
        &mut self,
        level: usize,
        use_checkpoint: bool,
        restored_time: Option<f32>,
        restored_coins: Option<u32>,
    ) {
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
    fn load_versus_map(&mut self) {
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
    #[inline]
    fn is_nearby_for_collision(
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
    fn check_player_platform_collisions(
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
    fn check_enemy_platform_collisions(enemy: &mut Enemy, platforms: &[Platform]) {
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
    fn calculate_versus_points(streak: u32) -> u32 {
        if streak == 0 {
            return 200;
        }
        let exp = streak.saturating_sub(1).min(10);
        let multiplier = 1u32.checked_shl(exp).unwrap_or(u32::MAX);
        200u32.saturating_mul(multiplier)
    }
    fn is_player_on_platform(
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
    fn ensure_player_grounded(player: &mut Player, platforms: &[Platform]) {
        if player.on_ground && player.vel_y == 0.0 {
            let (px, py, pw, _ph) = player.get_rect();
            if let Some(platform_y) = Self::is_player_on_platform(px, py, pw, platforms) {
                player.y = platform_y - PLAYER_HEIGHT;
            } else {
                player.y = GROUND_Y - PLAYER_HEIGHT;
            }
        }
    }
    fn update_versus_player_physics(
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
    pub fn update(&mut self, dt: f32) {
        self.update_transition(dt);

        if self.error_timer > 0.0 {
            self.error_timer -= dt;
            if self.error_timer <= 0.0 {
                self.error_message = None;
            }
        }
        self.particles.iter_mut().for_each(|p| p.4 -= dt);
        self.particles.retain(|p| p.4 > 0.0);
        let max_particles = PARTICLE_COUNT * 20;
        if self.particles.len() > max_particles {
            let excess = self.particles.len() - max_particles;
            self.particles.drain(0..excess);
        }

        if matches!(self.state, GameState::Menu) {
            self.save_check_timer += dt;
            if self.save_check_timer >= SAVE_CHECK_INTERVAL {
                self.check_for_new_saves();
                self.save_check_timer = 0.0;
            }
        } else {
            self.save_check_timer = 0.0;
        }

        if self.transition_target_state.is_some() && !matches!(self.state, GameState::Menu) {
            return;
        }

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
            GameState::Menu => {
                self.menu_animation_time += dt * MENU_ANIMATION_SPEED;
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if self.menu_selection > 0 {
                        self.menu_selection -= 1;
                        self.menu_animation_time = 0.0;
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if self.menu_selection < 6 {
                        self.menu_selection += 1;
                        self.menu_animation_time = 0.0;
                        self.audio.play_menu_select();
                    }
                }
                if self.previous_menu_selection != self.menu_selection {
                    self.previous_menu_selection = self.menu_selection;
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.audio.play_menu_select();
                    match self.menu_selection {
                        0 => {
                            self.has_new_save = false;
                            self.start_transition(GameState::ContinueMenu);
                            self.continue_selection = 0;
                            self.continue_mode = ContinueMode::View;
                        }
                        1 => {
                            self.start_transition(GameState::NameInput);
                            self.name_input.clear();
                            self.name_input_error = None;
                        }
                        2 => {
                            self.start_transition(GameState::LevelSelect);
                            self.level_selection = 0;
                        }
                        3 => {
                            self.versus_played = true;
                            self.load_versus_map();
                            self.start_transition(GameState::Versus);
                        }
                        4 => {
                            self.state = GameState::Settings;
                            self.settings_selection = 0;
                        }
                        5 => {
                            self.state = GameState::Credits;
                        }
                        6 => {
                            self.state = GameState::MenuExitConfirm;
                            self.menu_selection = 0;
                        }
                        _ => {}
                    }
                }
            }
            GameState::MenuExitConfirm => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace) {
                    self.audio.play_menu_select();
                    self.state = GameState::Menu;
                }
            }
            GameState::NameInput => {
                if let Some(ch) = get_char_pressed() {
                    if ch.is_alphanumeric() || ch == ' ' || ch == '-' || ch == '_' {
                        if self.name_input.len() < 20 {
                            self.name_input.push(ch);
                            self.name_input_error = None;
                        }
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    self.name_input.pop();
                    self.name_input_error = None;
                }
                let (is_valid, error_msg) = name_filter::is_name_valid(&self.name_input);
                if !is_valid && !self.name_input.is_empty() {
                    self.name_input_error = error_msg;
                } else {
                    self.name_input_error = None;
                }
                if is_key_pressed(KeyCode::Enter) {
                    let (is_valid, _) = name_filter::is_name_valid(&self.name_input);
                    if is_valid {
                        self.player_name = self.name_input.clone();
                        if self.player_name.to_lowercase() == "guicybercode" {
                            self.lives = EASTER_EGG_LIVES;
                        }
                        self.state = GameState::LevelSelect;
                        self.level_selection = 0;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.name_input.clear();
                    self.name_input_error = None;
                    self.state = GameState::Menu;
                }
            }
            GameState::ContinueMenu => {
                if self.continue_mode == ContinueMode::View {
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                        if self.continue_selection > 0 {
                            self.continue_selection -= 1;
                            self.audio.play_menu_select();
                        }
                    }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                        if self.continue_selection < 2 {
                            self.continue_selection += 1;
                            self.audio.play_menu_select();
                        }
                    }
                    if is_key_pressed(KeyCode::Enter) {
                        let path = SaveData::get_save_path(self.continue_selection);
                        if SaveData::save_exists(&path) {
                            if let Err(e) = self.load_game(self.continue_selection) {
                                let error_msg = format!("Erro ao carregar save: {}", e);
                                eprintln!("{}", error_msg);
                                self.show_error(error_msg);
                            } else {
                                self.load_level(
                                    self.current_level,
                                    self.last_checkpoint_pos.is_some(),
                                    Some(self.time_remaining),
                                    Some(self.coins_collected),
                                );
                                self.start_transition(GameState::Playing);
                            }
                        }
                    }
                    if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
                        let path = SaveData::get_save_path(self.continue_selection);
                        if SaveData::save_exists(&path) {
                            self.continue_mode = ContinueMode::DeleteConfirm;
                        }
                    }
                    if is_key_pressed(KeyCode::Escape) {
                        self.state = GameState::Menu;
                    }
                } else {
                    if is_key_pressed(KeyCode::Y) {
                        if let Err(e) = SaveData::delete_save(self.continue_selection) {
                            let error_msg = format!("Erro ao apagar save: {}", e);
                            eprintln!("{}", error_msg);
                            self.show_error(error_msg);
                        }
                        self.continue_mode = ContinueMode::View;
                    }
                    if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
                        self.continue_mode = ContinueMode::View;
                    }
                }
            }
            GameState::Settings => {
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if self.settings_selection > 0 {
                        self.settings_selection -= 1;
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if self.settings_selection < 7 {
                        self.settings_selection += 1;
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    match self.settings_selection {
                        0 => {
                            self.sound_enabled = !self.sound_enabled;
                            self.audio.set_enabled(self.sound_enabled);
                            self.audio.play_menu_select();
                        }
                        1 => {
                            if self.resolution_index > 0 {
                                self.resolution_index -= 1;
                                self.apply_resolution();
                                self.audio.play_menu_select();
                            }
                        }
                        3 => {
                            self.difficulty_multiplier = match self.difficulty_multiplier {
                                x if x == DIFFICULTY_EASY => DIFFICULTY_NORMAL,
                                x if x == DIFFICULTY_NORMAL => DIFFICULTY_HARD,
                                x if x == DIFFICULTY_HARD => DIFFICULTY_INSANE,
                                _ => DIFFICULTY_EASY,
                            };
                            self.audio.play_menu_select();
                        }
                        4 => {
                            self.colorblind_mode = !self.colorblind_mode;
                            self.audio.play_menu_select();
                        }
                        5 => {
                            self.font_size_scale = match self.font_size_scale {
                                1.0 => 1.25,
                                1.25 => 1.5,
                                1.5 => 0.75,
                                _ => 1.0,
                            };
                            self.audio.play_menu_select();
                        }
                        6 => {
                            self.assist_mode = !self.assist_mode;
                            self.audio.play_menu_select();
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    match self.settings_selection {
                        0 => {
                            self.sound_enabled = !self.sound_enabled;
                            self.audio.set_enabled(self.sound_enabled);
                            self.audio.play_menu_select();
                        }
                        1 => {
                            if self.resolution_index < self.available_resolutions.len() - 1 {
                                self.resolution_index += 1;
                                self.apply_resolution();
                                self.audio.play_menu_select();
                            }
                        }
                        3 => {
                            self.difficulty_multiplier = match self.difficulty_multiplier {
                                x if x == DIFFICULTY_EASY => DIFFICULTY_NORMAL,
                                x if x == DIFFICULTY_NORMAL => DIFFICULTY_HARD,
                                x if x == DIFFICULTY_HARD => DIFFICULTY_INSANE,
                                _ => DIFFICULTY_EASY,
                            };
                            self.audio.play_menu_select();
                        }
                        4 => {
                            self.colorblind_mode = !self.colorblind_mode;
                            self.audio.play_menu_select();
                        }
                        5 => {
                            self.font_size_scale = match self.font_size_scale {
                                1.0 => 1.25,
                                1.25 => 1.5,
                                1.5 => 0.75,
                                _ => 1.0,
                            };
                            self.audio.play_menu_select();
                        }
                        6 => {
                            self.assist_mode = !self.assist_mode;
                            self.audio.play_menu_select();
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    match self.settings_selection {
                        0 => {
                            self.sound_enabled = !self.sound_enabled;
                            self.audio.set_enabled(self.sound_enabled);
                            self.audio.play_menu_select();
                        }
                        2 => {
                            self.state = GameState::Controls;
                            self.controls_selection = 0;
                            self.controls_player = 1;
                            self.controls_waiting_input = None;
                        }
                        7 => {
                            self.audio.play_menu_select();
                            if self.came_from_pause {
                                self.came_from_pause = false;
                                self.state = GameState::Pause;
                            } else {
                                self.state = GameState::Menu;
                                self.menu_selection = 0;
                            }
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.audio.play_menu_select();
                    if self.came_from_pause {
                        self.came_from_pause = false;
                        self.state = GameState::Pause;
                    } else {
                        self.state = GameState::Menu;
                        self.menu_selection = 0;
                    }
                }
            }
            GameState::Controls => {
                if let Some((player, action)) = self.controls_waiting_input {
                    let mut _captured = false;
                    let keys_to_check = [
                        KeyCode::A,
                        KeyCode::B,
                        KeyCode::C,
                        KeyCode::D,
                        KeyCode::E,
                        KeyCode::F,
                        KeyCode::G,
                        KeyCode::H,
                        KeyCode::I,
                        KeyCode::J,
                        KeyCode::K,
                        KeyCode::L,
                        KeyCode::M,
                        KeyCode::N,
                        KeyCode::O,
                        KeyCode::P,
                        KeyCode::Q,
                        KeyCode::R,
                        KeyCode::S,
                        KeyCode::T,
                        KeyCode::U,
                        KeyCode::V,
                        KeyCode::W,
                        KeyCode::X,
                        KeyCode::Y,
                        KeyCode::Z,
                        KeyCode::Space,
                        KeyCode::Enter,
                        KeyCode::Escape,
                        KeyCode::Left,
                        KeyCode::Right,
                        KeyCode::Up,
                        KeyCode::Down,
                        KeyCode::Key0,
                        KeyCode::Key1,
                        KeyCode::Key2,
                        KeyCode::Key3,
                        KeyCode::Key4,
                        KeyCode::Key5,
                        KeyCode::Key6,
                        KeyCode::Key7,
                        KeyCode::Key8,
                        KeyCode::Key9,
                    ];
                    for &keycode in &keys_to_check {
                        if is_key_pressed(keycode) {
                            let controls = if player == 1 {
                                &mut self.player1_controls
                            } else {
                                &mut self.player2_controls
                            };
                            match action {
                                ControlAction::Left => controls.left = Some(keycode),
                                ControlAction::Right => controls.right = Some(keycode),
                                ControlAction::Jump => controls.jump = Some(keycode),
                            }
                            self.controls_waiting_input = None;
                            self.audio.play_menu_select();
                            _captured = true;
                            break;
                        }
                    }
                    if is_key_pressed(KeyCode::Escape) {
                        self.controls_waiting_input = None;
                        self.audio.play_menu_select();
                    }
                } else {
                    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                        if self.controls_selection > 0 {
                            self.controls_selection -= 1;
                            self.audio.play_menu_select();
                        }
                    }
                    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                        if self.controls_selection < 2 {
                            self.controls_selection += 1;
                            self.audio.play_menu_select();
                        }
                    }
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                        if self.controls_player == 2 {
                            self.controls_player = 1;
                            self.controls_selection = 0;
                            self.audio.play_menu_select();
                        }
                    }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                        if self.controls_player == 1 {
                            self.controls_player = 2;
                            self.controls_selection = 0;
                            self.audio.play_menu_select();
                        }
                    }
                    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                        let action = match self.controls_selection {
                            0 => ControlAction::Left,
                            1 => ControlAction::Right,
                            2 => ControlAction::Jump,
                            _ => return,
                        };
                        self.controls_waiting_input = Some((self.controls_player, action));
                        self.audio.play_menu_select();
                    }
                    if is_key_pressed(KeyCode::Escape) {
                        self.state = GameState::Settings;
                        self.controls_waiting_input = None;
                        self.audio.play_menu_select();
                    }
                }
            }
            GameState::Credits => {
                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                {
                    if self.came_from_pause {
                        self.came_from_pause = false;
                        self.state = GameState::Pause;
                    } else {
                        self.state = GameState::Menu;
                        self.menu_selection = 0;
                    }
                }
            }
            GameState::Tutorial => {
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    if self.tutorial_page > 0 {
                        self.tutorial_page -= 1;
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    if self.tutorial_page < TUTORIAL_PAGE_COUNT - 1 {
                        self.tutorial_page += 1;
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    if self.tutorial_page == TUTORIAL_PAGE_COUNT - 1 {
                        self.tutorial_completed = true;
                        self.start_transition(GameState::LevelSelect);
                    } else {
                        if self.tutorial_page < TUTORIAL_PAGE_COUNT - 1 {
                            self.tutorial_page += 1;
                        }
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.start_transition(GameState::LevelSelect);
                }
            }
            GameState::LevelSelect => {
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    for i in (0..self.level_selection).rev() {
                        if self.unlocked_levels[i] {
                            self.level_selection = i;
                            break;
                        }
                    }
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    for i in (self.level_selection + 1)..self.unlocked_levels.len() {
                        if self.unlocked_levels[i] {
                            self.level_selection = i;
                            break;
                        }
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    if self.level_selection < self.unlocked_levels.len()
                        && self.unlocked_levels[self.level_selection]
                    {
                        if self.level_selection == 0 && !self.tutorial_completed {
                            self.start_transition(GameState::Tutorial);
                        } else {
                            self.last_checkpoint_pos = None;
                            self.load_level(self.level_selection + 1, false, None, None);
                            self.score = 0;
                            if matches!(self.state, GameState::LevelSelect) {
                                let coming_from_coop = self.menu_selection == 2;
                                if coming_from_coop {
                                    self.player2 = Some(Player::new(
                                        150.0,
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
                                    }
                                    self.start_transition(GameState::Coop);
                                } else {
                                    self.start_transition(GameState::Playing);
                                }
                            } else {
                                self.start_transition(GameState::Playing);
                            }
                        }
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Menu;
                    self.menu_selection = 0;
                }
            }
            GameState::Playing => {
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
                let p1_right =
                    self.is_control_pressed(&self.player1_controls, ControlAction::Right);
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
                }
                self.player.update_animation(dt);
                if self.player.on_ground && self.player.vel_x.abs() > MIN_VELOCITY_FOR_FOOTSTEP {
                    self.footstep_timer += effective_dt;
                    if self.footstep_timer >= FOOTSTEP_INTERVAL {
                        self.audio.play_footstep(self.is_easter_egg());
                        self.footstep_timer = 0.0;
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
                            break;
                        }
                        Some(false) => {
                            self.audio.play_enemy_death();
                            self.score += SCORE_ENEMY;
                            self.player.vel_y = JUMP_FORCE * JUMP_BOUNCE_MULTIPLIER;
                        }
                        None => {}
                    }
                }
                for coin in &mut self.coins {
                    if coin.collected {
                        continue;
                    }
                    coin.update(effective_dt);
                    if coin.check_collection(px, py, pw, ph) {
                        self.coins_collected += 1;
                        self.score += SCORE_COIN;
                        self.audio.play_coin();
                        for _ in 0..PARTICLE_COUNT / 2 {
                            let angle = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
                            let speed = rand::gen_range(30.0, 80.0);
                            self.particles.push((
                                coin.x + COIN_SIZE / 2.0,
                                coin.y + COIN_SIZE / 2.0,
                                angle.cos() * speed,
                                angle.sin() * speed,
                                PARTICLE_LIFETIME,
                            ));
                        }
                    }
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
                        let error_msg = format!("Erro ao salvar jogo: {}", e);
                        eprintln!("{}", error_msg);
                        self.show_error(error_msg);
                    }
                    self.state = GameState::LevelComplete;
                }
                let screen_width = screen_width();
                let shake = if self.camera_shake_timer > 0.0 {
                    let random = ((get_time() * 1000.0) as u32 % 100) as f32 / 100.0;
                    (random - 0.5) * self.camera_shake_intensity
                } else {
                    0.0
                };
                self.camera.update(self.player.x, screen_width, shake);
            }
            GameState::LevelComplete => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.state = GameState::LevelSelect;
                    if self.current_level < MAX_LEVELS
                        && self.current_level < self.unlocked_levels.len()
                        && self.unlocked_levels[self.current_level]
                    {
                        self.level_selection = self.current_level;
                    } else if self.current_level > 0 {
                        self.level_selection = self.current_level - 1;
                    } else {
                        self.level_selection = 0;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::LevelSelect;
                    self.level_selection = self.current_level - 1;
                }
            }
            GameState::Versus => {
                if is_key_pressed(KeyCode::Escape) {
                    self.transition_to_menu();
                    self.menu_selection = 0;
                    self.player2 = None;
                    return;
                }
                let effective_dt = if self.assist_mode {
                    dt * ASSIST_MODE_SLOW_MOTION
                } else {
                    dt
                };
                self.versus_time_remaining -= effective_dt;
                if self.versus_time_remaining <= 0.0 {
                    self.versus_time_remaining = 0.0;
                    self.audio.play_level_complete();
                    self.state = GameState::VersusEnd;
                    return;
                }
                if self.respawn_timer_p1 > 0.0 {
                    self.respawn_timer_p1 -= dt;
                    if self.respawn_timer_p1 <= 0.0 {
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
                    }
                }
                if self.respawn_timer_p2 > 0.0 {
                    self.respawn_timer_p2 -= dt;
                    if let Some(ref mut p2) = self.player2 {
                        if self.respawn_timer_p2 <= 0.0 {
                            *p2 = Player::new(
                                700.0,
                                GROUND_Y - PLAYER_HEIGHT,
                                self.player_sprite_texture_p1
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                                self.player_sprite_texture_p2
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                            );
                            p2.on_ground = true;
                            p2.vel_y = 0.0;
                            p2.y = GROUND_Y - PLAYER_HEIGHT;
                        }
                    }
                }
                if self.respawn_timer_p1 <= 0.0 {
                    let p1_left =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Left);
                    let p1_right =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Right);
                    let p1_jump =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Jump);
                    let use_easter_egg = self.is_easter_egg();
                    let jumped = Self::update_versus_player_physics(
                        &mut self.player,
                        p1_left,
                        p1_right,
                        p1_jump,
                        &self.versus_platforms,
                        effective_dt,
                    );
                    if jumped {
                        self.audio.play_jump(use_easter_egg);
                    }
                    if self.player.on_ground && self.player.vel_x.abs() > MIN_VELOCITY_FOR_FOOTSTEP
                    {
                        self.footstep_timer += effective_dt;
                        if self.footstep_timer >= FOOTSTEP_INTERVAL {
                            self.audio.play_footstep(use_easter_egg);
                            self.footstep_timer = 0.0;
                        }
                    } else {
                        self.footstep_timer = 0.0;
                    }
                }
                let p2_controls = self.player2_controls.clone();
                let p2_left = self.is_control_pressed(&p2_controls, ControlAction::Left);
                let p2_right = self.is_control_pressed(&p2_controls, ControlAction::Right);
                let p2_jump = self.is_control_pressed(&p2_controls, ControlAction::Jump);
                if let Some(ref mut p2) = self.player2 {
                    if self.respawn_timer_p2 <= 0.0 {
                        let jumped = Self::update_versus_player_physics(
                            p2,
                            p2_left,
                            p2_right,
                            p2_jump,
                            &self.versus_platforms,
                            effective_dt,
                        );
                        if jumped {
                            self.audio.play_jump(false);
                        }
                    } else {
                        p2.update(effective_dt);
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        Self::check_player_platform_collisions(
                            p2,
                            &self.versus_platforms,
                            (px2, py2, pw2, ph2),
                        );
                    }
                }
                if self.respawn_timer_p1 > 0.0 {
                    self.player.update(effective_dt);
                    let (px, py, pw, ph) = self.player.get_rect();
                    Self::check_player_platform_collisions(
                        &mut self.player,
                        &self.versus_platforms,
                        (px, py, pw, ph),
                    );
                }
                if self.respawn_timer_p1 <= 0.0 && self.respawn_timer_p2 <= 0.0 {
                    if let Some(ref mut p2) = self.player2 {
                        if self.player.check_stomp(p2, self.player.vel_y) {
                            self.player1_score += 1;
                            self.player1_streak += 1;
                            self.player2_streak = 0;
                            let points = Self::calculate_versus_points(self.player1_streak);
                            self.player1_points += points;
                            self.audio.play_enemy_death();
                            self.respawn_timer_p2 = 2.0;
                            self.player.vel_y = JUMP_FORCE * JUMP_BOUNCE_MULTIPLIER;
                        } else if p2.check_stomp(&self.player, p2.vel_y) {
                            self.player2_score += 1;
                            self.player2_streak += 1;
                            self.player1_streak = 0;
                            let points = Self::calculate_versus_points(self.player2_streak);
                            self.player2_points += points;
                            self.audio.play_enemy_death();
                            self.respawn_timer_p1 = 2.0;
                            p2.vel_y = JUMP_FORCE * 0.6;
                        }
                    }
                }
                let p1_left = self.player.x;
                let p1_right = self.player.x + self.player.width;
                if p1_left < 0.0 {
                    self.player.x = 0.0;
                    self.player.vel_x = 0.0;
                }
                if p1_right > WORLD_WIDTH {
                    self.player.x = WORLD_WIDTH - self.player.width;
                    self.player.vel_x = 0.0;
                }
                if let Some(ref mut p2) = self.player2 {
                    let p2_left = p2.x;
                    let p2_right = p2.x + p2.width;
                    if p2_left < 0.0 {
                        p2.x = 0.0;
                        p2.vel_x = 0.0;
                    }
                    if p2_right > WORLD_WIDTH {
                        p2.x = WORLD_WIDTH - p2.width;
                        p2.vel_x = 0.0;
                    }
                }
                if self.player.y > FALL_DEATH_Y && self.respawn_timer_p1 <= 0.0 {
                    self.player2_score += 1;
                    self.player2_streak += 1;
                    self.player1_streak = 0;
                    let points = Self::calculate_versus_points(self.player2_streak);
                    self.player2_points += points;
                    self.audio.play_enemy_death();
                    self.respawn_timer_p1 = 2.0;
                }
                if let Some(ref p2) = self.player2 {
                    if p2.y > FALL_DEATH_Y && self.respawn_timer_p2 <= 0.0 {
                        self.player1_score += 1;
                        self.player1_streak += 1;
                        self.player2_streak = 0;
                        let points = Self::calculate_versus_points(self.player1_streak);
                        self.player1_points += points;
                        self.audio.play_enemy_death();
                        self.respawn_timer_p2 = 2.0;
                    }
                }
                if let Some(ref p2) = self.player2 {
                    let center_x = (self.player.x + p2.x) / 2.0;
                    let screen_width = screen_width();
                    let shake = if self.camera_shake_timer > 0.0 {
                        let random = ((get_time() * 1000.0) as u32 % 100) as f32 / 100.0;
                        (random - 0.5) * self.camera_shake_intensity
                    } else {
                        0.0
                    };
                    self.camera.update(center_x, screen_width, shake);
                }
            }
            GameState::VersusEnd => {
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Escape)
                {
                    self.state = GameState::Menu;
                    self.menu_selection = 0;
                    self.player2 = None;
                }
            }
            GameState::Coop => {
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
                    self.player2 = None;
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
                if self.respawn_timer_p1 > 0.0 {
                    self.respawn_timer_p1 -= dt;
                    if self.respawn_timer_p1 <= 0.0 {
                        if let Some(checkpoint) = self.last_checkpoint_pos {
                            self.player = Player::new(
                                checkpoint.0,
                                checkpoint.1 - PLAYER_HEIGHT,
                                self.player_sprite_texture_p1
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                                self.player_sprite_texture_p2
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                            );
                        } else {
                            self.player = Player::new(
                                50.0,
                                GROUND_Y - PLAYER_HEIGHT,
                                self.player_sprite_texture_p1
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                                self.player_sprite_texture_p2
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                            );
                        }
                        self.player.on_ground = true;
                        self.player.vel_y = 0.0;
                    }
                }
                if self.respawn_timer_p2 > 0.0 {
                    self.respawn_timer_p2 -= dt;
                    if let Some(ref mut p2) = self.player2 {
                        if self.respawn_timer_p2 <= 0.0 {
                            if let Some(checkpoint) = self.last_checkpoint_pos {
                                *p2 = Player::new(
                                    checkpoint.0 + 100.0,
                                    checkpoint.1 - PLAYER_HEIGHT,
                                    self.player_sprite_texture_p1
                                        .as_ref()
                                        .map(|t| std::rc::Rc::clone(t)),
                                    self.player_sprite_texture_p2
                                        .as_ref()
                                        .map(|t| std::rc::Rc::clone(t)),
                                );
                            } else {
                                *p2 = Player::new(
                                    150.0,
                                    GROUND_Y - PLAYER_HEIGHT,
                                    self.player_sprite_texture_p1
                                        .as_ref()
                                        .map(|t| std::rc::Rc::clone(t)),
                                    self.player_sprite_texture_p2
                                        .as_ref()
                                        .map(|t| std::rc::Rc::clone(t)),
                                );
                            }
                            p2.on_ground = true;
                            p2.vel_y = 0.0;
                        }
                    }
                }
                if self.respawn_timer_p1 <= 0.0 {
                    let p1_left =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Left);
                    let p1_right =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Right);
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
                    let p1_jump =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Jump);
                    let jumped = self.player.handle_jump_custom(p1_jump);
                    if jumped {
                        self.audio.play_jump(self.is_easter_egg());
                    }
                    self.player.update_animation(dt);
                    if self.player.on_ground && self.player.vel_x.abs() > MIN_VELOCITY_FOR_FOOTSTEP
                    {
                        self.footstep_timer += effective_dt;
                        if self.footstep_timer >= FOOTSTEP_INTERVAL {
                            self.audio.play_footstep(self.is_easter_egg());
                            self.footstep_timer = 0.0;
                        }
                    } else {
                        self.footstep_timer = 0.0;
                    }
                }
                let p2_controls = self.player2_controls.clone();
                let p2_left = self.is_control_pressed(&p2_controls, ControlAction::Left);
                let p2_right = self.is_control_pressed(&p2_controls, ControlAction::Right);
                let p2_jump = self.is_control_pressed(&p2_controls, ControlAction::Jump);
                if let Some(ref mut p2) = self.player2 {
                    if self.respawn_timer_p2 <= 0.0 {
                        p2.handle_movement_custom(p2_left, p2_right);
                        p2.update(effective_dt);
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        for checkpoint in &mut self.checkpoints {
                            if checkpoint.check_activation(px2, py2, pw2, ph2) {
                                self.last_checkpoint_pos = Some((checkpoint.x, checkpoint.y));
                                self.score += SCORE_CHECKPOINT;
                                self.audio.play_coin();
                            }
                        }
                        Self::check_player_platform_collisions(
                            p2,
                            &self.platforms,
                            (px2, py2, pw2, ph2),
                        );
                        let jumped = p2.handle_jump_custom(p2_jump);
                        if jumped {
                            self.audio.play_jump(false);
                        }
                        p2.update_animation(effective_dt);
                    } else {
                        p2.update(effective_dt);
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        Self::check_player_platform_collisions(
                            p2,
                            &self.platforms,
                            (px2, py2, pw2, ph2),
                        );
                    }
                }
                if self.respawn_timer_p1 > 0.0 {
                    self.player.update(effective_dt);
                    let (px, py, pw, ph) = self.player.get_rect();
                    Self::check_player_platform_collisions(
                        &mut self.player,
                        &self.platforms,
                        (px, py, pw, ph),
                    );
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
                if let Some(ref mut p2) = self.player2 {
                    let p2_left = p2.x;
                    let p2_right = p2.x + p2.width;
                    if p2_left < 0.0 {
                        p2.x = 0.0;
                        p2.vel_x = 0.0;
                    }
                    if p2_right > WORLD_WIDTH {
                        p2.x = WORLD_WIDTH - p2.width;
                        p2.vel_x = 0.0;
                    }
                }
                if self.player.y > FALL_DEATH_Y && self.respawn_timer_p1 <= 0.0 {
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
                if let Some(ref p2) = self.player2 {
                    if p2.y > FALL_DEATH_Y && self.respawn_timer_p2 <= 0.0 {
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
                }
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
                    let (px, py, pw, ph) = self.player.get_rect();
                    match enemy.check_player_collision(px, py, pw, ph, self.player.vel_y) {
                        Some(true) => {
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
                            break;
                        }
                        Some(false) => {
                            self.audio.play_enemy_death();
                            self.score += SCORE_ENEMY;
                            self.player.vel_y = JUMP_FORCE * JUMP_BOUNCE_MULTIPLIER;
                        }
                        None => {}
                    }
                    if let Some(ref p2) = self.player2 {
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        match enemy.check_player_collision(px2, py2, pw2, ph2, p2.vel_y) {
                            Some(true) => {
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
                                break;
                            }
                            Some(false) => {
                                self.audio.play_enemy_death();
                                self.score += SCORE_ENEMY;
                                if let Some(ref mut p2) = self.player2 {
                                    p2.vel_y = JUMP_FORCE * JUMP_BOUNCE_MULTIPLIER;
                                }
                            }
                            None => {}
                        }
                    }
                }
                for coin in &mut self.coins {
                    if coin.collected {
                        continue;
                    }
                    coin.update(effective_dt);
                    let (px, py, pw, ph) = self.player.get_rect();
                    if coin.check_collection(px, py, pw, ph) {
                        self.coins_collected += 1;
                        self.score += SCORE_COIN;
                        self.audio.play_coin();
                    }
                    if let Some(ref p2) = self.player2 {
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        if coin.check_collection(px2, py2, pw2, ph2) {
                            self.coins_collected += 1;
                            self.score += SCORE_COIN;
                            self.audio.play_coin();
                        }
                    }
                }
                if (self.player.x > LEVEL_COMPLETE_X || self.coins_collected >= self.total_coins)
                    && (self.player2.is_none()
                        || self
                            .player2
                            .as_ref()
                            .map(|p| {
                                p.x > LEVEL_COMPLETE_X || self.coins_collected >= self.total_coins
                            })
                            .unwrap_or(true))
                {
                    let time_bonus = (self.time_remaining * SCORE_TIME_BONUS) as u32;
                    self.score += SCORE_LEVEL_COMPLETE + time_bonus;
                    self.audio.play_level_complete();
                    if self.current_level < MAX_LEVELS
                        && self.current_level < self.unlocked_levels.len()
                    {
                        self.unlocked_levels[self.current_level] = true;
                    }
                    if let Err(e) = self.save_game(0) {
                        let error_msg = format!("Erro ao salvar jogo: {}", e);
                        eprintln!("{}", error_msg);
                        self.show_error(error_msg);
                    }
                    self.state = GameState::LevelComplete;
                }
                if let Some(ref p2) = self.player2 {
                    let center_x = (self.player.x + p2.x) / 2.0;
                    let screen_w = screen_width();
                    let shake = if self.camera_shake_timer > 0.0 {
                        let random = ((get_time() * 1000.0) as u32 % 100) as f32 / 100.0;
                        (random - 0.5) * self.camera_shake_intensity
                    } else {
                        0.0
                    };
                    self.camera.update(center_x, screen_w, shake);
                } else {
                    let player_x = self.player.x;
                    let screen_w = screen_width();
                    let shake = if self.camera_shake_timer > 0.0 {
                        let random = ((get_time() * 1000.0) as u32 % 100) as f32 / 100.0;
                        (random - 0.5) * self.camera_shake_intensity
                    } else {
                        0.0
                    };
                    self.camera.update(player_x, screen_w, shake);
                }
            }
            GameState::Pause => {
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if self.pause_selection > 0 {
                        self.pause_selection -= 1;
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if self.pause_selection < 3 {
                        self.pause_selection += 1;
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.audio.play_menu_select();
                    match self.pause_selection {
                        0 => {
                            self.state = GameState::Playing;
                        }
                        1 => {
                            self.came_from_pause = true;
                            self.state = GameState::Settings;
                            self.settings_selection = 0;
                        }
                        2 => {
                            self.came_from_pause = true;
                            self.state = GameState::Credits;
                        }
                        3 => {
                            self.state = GameState::Menu;
                            self.menu_selection = 0;
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Playing;
                }
            }
            GameState::Respawn => {
                self.respawn_timer -= dt;
                if self.respawn_timer <= 0.0 {
                    self.respawn_timer = 0.0;
                    self.load_level(
                        self.current_level,
                        self.last_checkpoint_pos.is_some(),
                        Some(self.time_remaining),
                        Some(self.coins_collected),
                    );
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                if self.game_over_fade_timer > 0.0 {
                    self.game_over_fade_timer -= dt;
                }
                if self.game_over_fade_timer <= 0.0 {
                    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                        self.lives = DEFAULT_LIVES;
                        self.load_level(
                            self.current_level,
                            self.last_checkpoint_pos.is_some(),
                            None,
                            None,
                        );
                        self.state = GameState::Playing;
                    }
                    if is_key_pressed(KeyCode::Escape) {
                        self.state = GameState::LevelSelect;
                        self.level_selection = self.current_level - 1;
                    }
                }
            }
        }
    }
    fn draw_level_world(&self) {
        let screen_left = self.camera.x - COLLISION_MARGIN;
        let screen_right = self.camera.x + screen_width() + COLLISION_MARGIN;
        let screen_top = self.camera.y - COLLISION_MARGIN;
        let screen_bottom = self.camera.y + screen_height() + COLLISION_MARGIN;
        for platform in &self.platforms {
            if platform.x + platform.width >= screen_left
                && platform.x <= screen_right
                && platform.y + platform.height >= screen_top
                && platform.y <= screen_bottom
            {
                platform.draw(self.camera.x, self.camera.y);
            }
        }
        for checkpoint in &self.checkpoints {
            if checkpoint.x >= screen_left
                && checkpoint.x <= screen_right
                && checkpoint.y >= screen_top
                && checkpoint.y <= screen_bottom
            {
                checkpoint.draw(self.camera.x, self.camera.y);
            }
        }
        for coin in &self.coins {
            if !coin.collected
                && coin.x >= screen_left
                && coin.x <= screen_right
                && coin.y >= screen_top
                && coin.y <= screen_bottom
            {
                coin.draw(self.camera.x, self.camera.y);
            }
        }
        for enemy in &self.enemies {
            if enemy.alive
                && enemy.x >= screen_left
                && enemy.x <= screen_right
                && enemy.y >= screen_top
                && enemy.y <= screen_bottom
            {
                enemy.draw(self.camera.x, self.camera.y);
            }
        }
        self.player.draw(self.camera.x, self.camera.y);
        for particle in &self.particles {
            if particle.4 > 0.0 {
                let screen_x = particle.0 - self.camera.x;
                let screen_y = particle.1 - self.camera.y;
                let alpha = particle.4 / PARTICLE_LIFETIME;
                let color = if self.colorblind_mode {
                    Color::new(0.5, 0.5, 0.5, alpha)
                } else {
                    Color::new(1.0, 0.84, 0.0, alpha)
                };
                draw_circle(screen_x, screen_y, 3.0, color);
            }
        }
    }
    fn draw_level_hud(&self, include_time_label: bool) {
        let font_scale = self.font_size_scale;
        let time_seconds = self.time_remaining as u32;
        let time_text = if include_time_label {
            format!("Time: {}s", time_seconds)
        } else {
            format!("{}", time_seconds)
        };
        let time_color = if self.colorblind_mode {
            if self.time_remaining < TIME_WARNING_RED {
                DARKGRAY
            } else if self.time_remaining < TIME_WARNING_YELLOW {
                GRAY
            } else {
                BLACK
            }
        } else {
            if self.time_remaining < TIME_WARNING_RED {
                RED
            } else if self.time_remaining < TIME_WARNING_YELLOW {
                YELLOW
            } else {
                BLACK
            }
        };
        let player_name_display = if self.player_name.is_empty() {
            "Player"
        } else {
            &self.player_name
        };
        draw_text(player_name_display, 10.0, 30.0, 24.0 * font_scale, BLACK);
        draw_text(
            &format!(
                "Level: {} | Coins: {}/{} | Time: {}s",
                self.current_level, self.coins_collected, self.total_coins, time_seconds
            ),
            10.0,
            60.0,
            30.0 * font_scale,
            BLACK,
        );
        let score_text = format!("Score: {}", self.score);
        draw_text(&score_text, 10.0, 100.0, 28.0 * font_scale, BLACK);
        let lives_text = format!("Lives: {}", self.lives);
        let lives_color = if self.colorblind_mode {
            if self.lives <= 1 {
                DARKGRAY
            } else {
                BLACK
            }
        } else {
            if self.lives <= 1 {
                RED
            } else {
                BLACK
            }
        };
        draw_text(&lives_text, 10.0, 130.0, 28.0 * font_scale, lives_color);
        let time_width = measure_text(&time_text, None, (40.0 * font_scale) as u16, 1.0).width;
        draw_text(
            &time_text,
            screen_width() - time_width - 20.0,
            40.0,
            40.0 * font_scale,
            time_color,
        );
    }
    pub fn draw(&self) {
        clear_background(WHITE);
        match self.state {
            GameState::Splash => {
                let progress = (self.splash_timer / SPLASH_DURATION).min(1.0);
                let fade_in = if progress < 0.5 {
                    progress * 2.0
                } else if progress > 0.7 {
                    1.0 - ((progress - 0.7) / 0.3)
                } else {
                    1.0
                };
                let title = "JUMP QUEST";
                let title_size = MENU_TITLE_SIZE * 1.2;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                let title_color = Color::new(0.0, 0.0, 0.0, fade_in);
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    screen_height() / 2.0,
                    title_size,
                    title_color,
                );
                let version_text = format!("v{}", GAME_VERSION);
                let version_size = MENU_VERSION_SIZE;
                let version_width =
                    measure_text(&version_text, None, version_size as u16, 1.0).width;
                let version_color = Color::new(0.5, 0.5, 0.5, fade_in * 0.7);
                draw_text(
                    &version_text,
                    screen_width() / 2.0 - version_width / 2.0,
                    screen_height() / 2.0 + 80.0,
                    version_size,
                    version_color,
                );
            }
            GameState::Menu => {
                let title = "JUMP QUEST";
                let title_width = measure_text(title, None, MENU_TITLE_SIZE as u16, 1.0).width;
                let title_color = if self.is_easter_egg() {
                    let time = get_time() as f32;
                    let glow = (time * 2.0).sin() * 0.2 + 0.8;
                    Color::new(0.85 * glow, 0.65 * glow, 0.13 * glow, 1.0)
                } else {
                    BLACK
                };
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    screen_height() / 2.0 - 180.0,
                    MENU_TITLE_SIZE,
                    title_color,
                );
                let menu_options = vec![
                    "CONTINUE", "PLAY", "CO-OP", "VERSUS", "SETTINGS", "CREDITS", "EXIT",
                ];
                let start_y = screen_height() / 2.0 - 40.0;
                for (i, option) in menu_options.iter().enumerate() {
                    let option_width =
                        measure_text(option, None, MENU_OPTION_SIZE as u16, 1.0).width;
                    let x = screen_width() / 2.0 - option_width / 2.0;
                    let y = start_y + (i as f32 * MENU_OPTION_SPACING);
                    let color = if i == self.menu_selection {
                        BLACK
                    } else {
                        DARKGRAY
                    };
                    let title_color = if self.is_easter_egg() {
                        Color::new(0.85, 0.65, 0.13, 1.0)
                    } else {
                        BLACK
                    };
                    if i == 0 && self.is_easter_egg() {
                        draw_text(
                            title,
                            screen_width() / 2.0 - title_width / 2.0,
                            screen_height() / 2.0 - 180.0,
                            MENU_TITLE_SIZE,
                            title_color,
                        );
                    }
                    if i == self.menu_selection {
                        let anim_time = self.menu_animation_time % (2.0 * std::f32::consts::PI);
                        let alpha = (anim_time.sin() * 0.4 + 0.6).clamp(0.3, 1.0);
                        let indicator_color = Color::new(0.0, 0.0, 0.0, alpha);
                        draw_text(
                            ">",
                            x - MENU_INDICATOR_OFFSET,
                            y,
                            MENU_OPTION_SIZE,
                            indicator_color,
                        );
                        draw_line(
                            x - 10.0,
                            y + 8.0,
                            x + option_width + 10.0,
                            y + 8.0,
                            2.0,
                            Color::new(0.0, 0.0, 0.0, alpha * 0.5),
                        );
                    }
                    draw_text(option, x, y, MENU_OPTION_SIZE, color);
                    if i == 0 && self.has_new_save {
                        let new_text = "NEW";
                        draw_text(new_text, x + option_width + 10.0, y, 20.0, RED);
                    }
                    if i == 2 && !self.versus_played {
                        let new_text = "NEW";
                        draw_text(new_text, x + option_width + 10.0, y, 20.0, GREEN);
                    }
                }
                let last_option_y =
                    start_y + ((menu_options.len() - 1) as f32 * MENU_OPTION_SPACING);
                let last_option_height = MENU_OPTION_SIZE;
                let instructions_y = last_option_y + last_option_height + 40.0;
                let instructions = "ARROWS/WASD: Navigate | ENTER/SPACE: Select";
                let inst_width =
                    measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    instructions_y,
                    MENU_INSTRUCTION_SIZE,
                    GRAY,
                );
                let version_text = format!("v{}", GAME_VERSION);
                draw_text(
                    &version_text,
                    10.0,
                    screen_height() - 20.0,
                    MENU_VERSION_SIZE,
                    LIGHTGRAY,
                );
            }
            GameState::NameInput => {
                clear_background(WHITE);
                let title = "ENTER YOUR NAME";
                let title_size = 48.0;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    screen_height() / 2.0 - 150.0,
                    title_size,
                    BLACK,
                );
                let input_label = "Name:";
                let input_size = 32.0;
                draw_text(
                    input_label,
                    screen_width() / 2.0 - 200.0,
                    screen_height() / 2.0 - 50.0,
                    input_size,
                    BLACK,
                );
                let name_display = if self.name_input.is_empty() {
                    "_"
                } else {
                    &self.name_input
                };
                draw_text(
                    name_display,
                    screen_width() / 2.0 - 100.0,
                    screen_height() / 2.0 - 50.0,
                    input_size,
                    BLACK,
                );
                if let Some(ref error) = self.name_input_error {
                    let error_width = measure_text(error, None, 20, 1.0).width;
                    draw_text(
                        error,
                        screen_width() / 2.0 - error_width / 2.0,
                        screen_height() / 2.0 + 20.0,
                        20.0,
                        RED,
                    );
                } else if !self.name_input.is_empty() {
                    let (is_valid, _) = name_filter::is_name_valid(&self.name_input);
                    if is_valid {
                        let valid_text = "Name is valid";
                        let valid_width = measure_text(valid_text, None, 20, 1.0).width;
                        draw_text(
                            valid_text,
                            screen_width() / 2.0 - valid_width / 2.0,
                            screen_height() / 2.0 + 20.0,
                            20.0,
                            GREEN,
                        );
                    } else {
                        let hint_text = "Name must be 3-20 characters";
                        let hint_width = measure_text(hint_text, None, 20, 1.0).width;
                        draw_text(
                            hint_text,
                            screen_width() / 2.0 - hint_width / 2.0,
                            screen_height() / 2.0 + 20.0,
                            20.0,
                            GRAY,
                        );
                    }
                }
                if self.name_input.to_lowercase() == "guicybercode" {
                    let special_text = "Special mode activated!";
                    let special_width = measure_text(special_text, None, 24, 1.0).width;
                    let time = get_time() as f32;
                    let glow = (time * 3.0).sin() * 0.3 + 0.7;
                    draw_text(
                        special_text,
                        screen_width() / 2.0 - special_width / 2.0,
                        screen_height() / 2.0 + 60.0,
                        24.0,
                        Color::new(0.85 * glow, 0.65 * glow, 0.13 * glow, 1.0),
                    );
                }
                let instructions = "ENTER: Confirm | ESC: Cancel";
                let inst_width =
                    measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 40.0,
                    MENU_INSTRUCTION_SIZE,
                    GRAY,
                );
            }
            GameState::Tutorial => {
                clear_background(WHITE);
                let title = "HOW TO PLAY";
                let title_size = 48.0;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    80.0,
                    title_size,
                    BLACK,
                );
                let content = match self.tutorial_page {
                    0 => vec![
                        "CONTROLS",
                        "",
                        "Move: ARROW KEYS or A/D",
                        "Jump: SPACE or W",
                        "Pause: ESC",
                    ],
                    1 => vec![
                        "OBJECTIVES",
                        "",
                        "Collect all coins",
                        "Avoid enemies",
                        "Reach the end flag",
                        "Complete before time runs out",
                    ],
                    2 => vec![
                        "CHECKPOINTS",
                        "",
                        "Touch checkpoints to save progress",
                        "If you die, respawn at last checkpoint",
                        "Checkpoints give bonus points",
                    ],
                    3 => vec![
                        "LIVES & TIME",
                        "",
                        "You start with 5 lives",
                        "Lose a life when you die",
                        "Game over when lives reach 0",
                        "Complete levels quickly for time bonus",
                    ],
                    4 => vec![
                        "READY TO PLAY?",
                        "",
                        "Press ENTER to start",
                        "or ESC to skip tutorial",
                    ],
                    _ => vec![],
                };
                let start_y = 180.0;
                let line_spacing = 35.0;
                for (i, line) in content.iter().enumerate() {
                    let line_width = measure_text(line, None, 28, 1.0).width;
                    let color = if i == 0 && !line.is_empty() {
                        BLACK
                    } else {
                        DARKGRAY
                    };
                    draw_text(
                        line,
                        screen_width() / 2.0 - line_width / 2.0,
                        start_y + (i as f32 * line_spacing),
                        28.0,
                        color,
                    );
                }
                let page_text = format!("{}/{}", self.tutorial_page + 1, TUTORIAL_PAGE_COUNT);
                let page_width = measure_text(&page_text, None, 24, 1.0).width;
                draw_text(
                    &page_text,
                    screen_width() / 2.0 - page_width / 2.0,
                    screen_height() - 100.0,
                    24.0,
                    GRAY,
                );
                let instructions = "LEFT/RIGHT: Navigate | ENTER: Next/Start | ESC: Skip";
                let inst_width =
                    measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 40.0,
                    MENU_INSTRUCTION_SIZE,
                    GRAY,
                );
            }
            GameState::ContinueMenu => {
                clear_background(WHITE);
                let title = "CONTINUE GAME";
                let title_size = 48.0;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    100.0,
                    title_size,
                    BLACK,
                );
                let saves = SaveData::list_all_saves();
                let start_y = 200.0;
                let slot_spacing = 80.0;
                for (slot_idx, (_, save_data_opt)) in saves.iter().enumerate() {
                    let y = start_y + (slot_idx as f32 * slot_spacing);
                    let slot_num = slot_idx + 1;
                    let color = if slot_idx == self.continue_selection {
                        BLACK
                    } else {
                        DARKGRAY
                    };
                    if slot_idx == self.continue_selection {
                        draw_text(">", 100.0, y, 32.0, BLACK);
                    }
                    if let Some(save_data) = save_data_opt {
                        let minutes = (save_data.time_taken / 60.0) as u32;
                        let seconds = (save_data.time_taken % 60.0) as u32;
                        let slot_info = format!(
                            "Slot {}: Level {} | Score: {} | Lives: {} | Time: {}:{:02} | Name: {}",
                            slot_num,
                            save_data.current_level,
                            save_data.score,
                            save_data.lives,
                            minutes,
                            seconds,
                            if save_data.player_name.is_empty() {
                                "Unknown"
                            } else {
                                &save_data.player_name
                            }
                        );
                        draw_text(&slot_info, 150.0, y, 24.0, color);
                    } else {
                        let empty_text = format!("Slot {}: Empty Slot", slot_num);
                        draw_text(&empty_text, 150.0, y, 24.0, color);
                    }
                }
                if self.continue_mode == ContinueMode::DeleteConfirm {
                    let confirm_text = "Are you sure you want to delete this save? (Y/N)";
                    let confirm_width = measure_text(confirm_text, None, 28, 1.0).width;
                    draw_text(
                        confirm_text,
                        screen_width() / 2.0 - confirm_width / 2.0,
                        screen_height() / 2.0 + 100.0,
                        28.0,
                        RED,
                    );
                } else {
                    let instructions = "ENTER: Load | DELETE: Erase | ESC: Back";
                    let inst_width =
                        measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                    draw_text(
                        instructions,
                        screen_width() / 2.0 - inst_width / 2.0,
                        screen_height() - 40.0,
                        MENU_INSTRUCTION_SIZE,
                        GRAY,
                    );
                }
            }
            GameState::MenuExitConfirm => {
                let confirm_text = "ARE YOU SURE YOU WANT TO EXIT?";
                let confirm_size = 42.0;
                let confirm_width =
                    measure_text(confirm_text, None, confirm_size as u16, 1.0).width;
                draw_text(
                    confirm_text,
                    screen_width() / 2.0 - confirm_width / 2.0,
                    screen_height() / 2.0 - 100.0,
                    confirm_size,
                    BLACK,
                );
                let yes_text = "YES (ENTER/SPACE)";
                let no_text = "NO (ESC)";
                let option_size = 32.0;
                let yes_width = measure_text(yes_text, None, option_size as u16, 1.0).width;
                let no_width = measure_text(no_text, None, option_size as u16, 1.0).width;
                draw_text(
                    yes_text,
                    screen_width() / 2.0 - yes_width / 2.0,
                    screen_height() / 2.0 - 20.0,
                    option_size,
                    RED,
                );
                draw_text(
                    no_text,
                    screen_width() / 2.0 - no_width / 2.0,
                    screen_height() / 2.0 + 40.0,
                    option_size,
                    DARKGRAY,
                );
            }
            GameState::Settings => {
                let title = "SETTINGS";
                let title_size = 48.0;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    80.0,
                    title_size,
                    BLACK,
                );
                let option_size = 30.0;
                let start_y = 180.0;
                let spacing = 60.0;
                let safe_resolution_index = self
                    .resolution_index
                    .min(self.available_resolutions.len().saturating_sub(1));
                let res_name = if safe_resolution_index < self.available_resolutions.len() {
                    let (w, h) = self.available_resolutions[safe_resolution_index];
                    format!("{}x{}", w, h)
                } else {
                    "Unknown".to_string()
                };
                let sound_text =
                    format!("SOUND: {}", if self.sound_enabled { "ON" } else { "OFF" });
                let sound_color = if self.settings_selection == 0 {
                    BLACK
                } else {
                    GRAY
                };
                let sound_width = measure_text(&sound_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 0 {
                    draw_text(
                        ">",
                        screen_width() / 2.0 - sound_width / 2.0 - 30.0,
                        start_y,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        "<",
                        screen_width() / 2.0 + sound_width / 2.0 + 10.0,
                        start_y,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &sound_text,
                    screen_width() / 2.0 - sound_width / 2.0,
                    start_y,
                    option_size,
                    sound_color,
                );
                let res_text = format!("RESOLUTION: {}", res_name);
                let res_color = if self.settings_selection == 1 {
                    BLACK
                } else {
                    GRAY
                };
                let res_width = measure_text(&res_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 1 {
                    draw_text(
                        "<",
                        screen_width() / 2.0 - res_width / 2.0 - 30.0,
                        start_y + spacing,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        ">",
                        screen_width() / 2.0 + res_width / 2.0 + 10.0,
                        start_y + spacing,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &res_text,
                    screen_width() / 2.0 - res_width / 2.0,
                    start_y + spacing,
                    option_size,
                    res_color,
                );
                let controls_text = "CONTROLS";
                let controls_color = if self.settings_selection == 2 {
                    BLACK
                } else {
                    GRAY
                };
                let controls_width =
                    measure_text(controls_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 2 {
                    draw_text(
                        ">",
                        screen_width() / 2.0 - controls_width / 2.0 - 30.0,
                        start_y + spacing * 2.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    controls_text,
                    screen_width() / 2.0 - controls_width / 2.0,
                    start_y + spacing * 2.0,
                    option_size,
                    controls_color,
                );
                if self.settings_selection == 2 {
                    let controls_info = [
                        "ARROWS / WASD - Move",
                        "SPACE / W / UP ARROW - Jump",
                        "ESC - Pause / Back",
                        "ENTER - Confirm",
                    ];
                    let info_size = 20.0;
                    let info_start_y = start_y + spacing * 2.5;
                    for (i, info) in controls_info.iter().enumerate() {
                        let info_width = measure_text(info, None, info_size as u16, 1.0).width;
                        draw_text(
                            info,
                            screen_width() / 2.0 - info_width / 2.0,
                            info_start_y + (i as f32 * 25.0),
                            info_size,
                            DARKGRAY,
                        );
                    }
                }
                let difficulty_name = match self.difficulty_multiplier {
                    x if x == DIFFICULTY_EASY => "EASY",
                    x if x == DIFFICULTY_NORMAL => "NORMAL",
                    x if x == DIFFICULTY_HARD => "HARD",
                    x if x == DIFFICULTY_INSANE => "INSANE",
                    _ => "NORMAL",
                };
                let difficulty_text = format!("DIFFICULTY: {}", difficulty_name);
                let difficulty_color = if self.settings_selection == 3 {
                    BLACK
                } else {
                    GRAY
                };
                let difficulty_width =
                    measure_text(&difficulty_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 3 {
                    draw_text(
                        "<",
                        screen_width() / 2.0 - difficulty_width / 2.0 - 30.0,
                        start_y + spacing * 3.0,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        ">",
                        screen_width() / 2.0 + difficulty_width / 2.0 + 10.0,
                        start_y + spacing * 3.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &difficulty_text,
                    screen_width() / 2.0 - difficulty_width / 2.0,
                    start_y + spacing * 3.0,
                    option_size,
                    difficulty_color,
                );
                let colorblind_text = format!(
                    "COLORBLIND MODE: {}",
                    if self.colorblind_mode { "ON" } else { "OFF" }
                );
                let colorblind_color = if self.settings_selection == 4 {
                    BLACK
                } else {
                    GRAY
                };
                let colorblind_width =
                    measure_text(&colorblind_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 4 {
                    draw_text(
                        ">",
                        screen_width() / 2.0 - colorblind_width / 2.0 - 30.0,
                        start_y + spacing * 4.0,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        "<",
                        screen_width() / 2.0 + colorblind_width / 2.0 + 10.0,
                        start_y + spacing * 4.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &colorblind_text,
                    screen_width() / 2.0 - colorblind_width / 2.0,
                    start_y + spacing * 4.0,
                    option_size,
                    colorblind_color,
                );
                let font_text = format!("FONT SIZE: {:.0}%", self.font_size_scale * 100.0);
                let font_color = if self.settings_selection == 5 {
                    BLACK
                } else {
                    GRAY
                };
                let font_width = measure_text(&font_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 5 {
                    draw_text(
                        "<",
                        screen_width() / 2.0 - font_width / 2.0 - 30.0,
                        start_y + spacing * 5.0,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        ">",
                        screen_width() / 2.0 + font_width / 2.0 + 10.0,
                        start_y + spacing * 5.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &font_text,
                    screen_width() / 2.0 - font_width / 2.0,
                    start_y + spacing * 5.0,
                    option_size,
                    font_color,
                );
                let assist_text = format!(
                    "ASSIST MODE: {}",
                    if self.assist_mode { "ON" } else { "OFF" }
                );
                let assist_color = if self.settings_selection == 6 {
                    BLACK
                } else {
                    GRAY
                };
                let assist_width = measure_text(&assist_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 6 {
                    draw_text(
                        ">",
                        screen_width() / 2.0 - assist_width / 2.0 - 30.0,
                        start_y + spacing * 6.0,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        "<",
                        screen_width() / 2.0 + assist_width / 2.0 + 10.0,
                        start_y + spacing * 6.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &assist_text,
                    screen_width() / 2.0 - assist_width / 2.0,
                    start_y + spacing * 6.0,
                    option_size,
                    assist_color,
                );
                let back_text = "BACK";
                let back_color = if self.settings_selection == 7 {
                    BLACK
                } else {
                    GRAY
                };
                let back_width = measure_text(back_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 7 {
                    draw_text(
                        ">",
                        screen_width() / 2.0 - back_width / 2.0 - 30.0,
                        start_y + spacing * 7.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    back_text,
                    screen_width() / 2.0 - back_width / 2.0,
                    start_y + spacing * 7.0,
                    option_size,
                    back_color,
                );
                let instructions =
                    "Use ARROWS to navigate and adjust, ENTER to confirm, ESC to go back";
                let inst_size = 16.0;
                let inst_width = measure_text(instructions, None, inst_size as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 40.0,
                    inst_size,
                    GRAY,
                );
            }
            GameState::Controls => {
                let title = "CONTROLS";
                let title_size = 48.0;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    80.0,
                    title_size,
                    BLACK,
                );
                if let Some((player, action)) = self.controls_waiting_input {
                    let waiting_text = format!(
                        "Press key for Player {} {}...",
                        player,
                        match action {
                            ControlAction::Left => "LEFT",
                            ControlAction::Right => "RIGHT",
                            ControlAction::Jump => "JUMP",
                        }
                    );
                    let waiting_width = measure_text(&waiting_text, None, 32, 1.0).width;
                    draw_text(
                        &waiting_text,
                        screen_width() / 2.0 - waiting_width / 2.0,
                        screen_height() / 2.0,
                        32.0,
                        RED,
                    );
                    let esc_text = "Press ESC to cancel";
                    let esc_width = measure_text(esc_text, None, 20, 1.0).width;
                    draw_text(
                        esc_text,
                        screen_width() / 2.0 - esc_width / 2.0,
                        screen_height() / 2.0 + 50.0,
                        20.0,
                        GRAY,
                    );
                } else {
                    let option_size = 28.0;
                    let start_y = 180.0;
                    let spacing = 50.0;
                    let player_text = format!("PLAYER {}", self.controls_player);
                    let player_color = BLACK;
                    let player_width = measure_text(&player_text, None, 36, 1.0).width;
                    draw_text(
                        &player_text,
                        screen_width() / 2.0 - player_width / 2.0,
                        start_y,
                        36.0,
                        player_color,
                    );
                    let controls = if self.controls_player == 1 {
                        &self.player1_controls
                    } else {
                        &self.player2_controls
                    };
                    let actions = [
                        ("LEFT", controls.left, controls.left_gamepad),
                        ("RIGHT", controls.right, controls.right_gamepad),
                        ("JUMP", controls.jump, controls.jump_gamepad),
                    ];
                    for (i, (action_name, key, gamepad)) in actions.iter().enumerate() {
                        let y = start_y + 80.0 + (i as f32 * spacing);
                        let color = if i == self.controls_selection {
                            BLACK
                        } else {
                            GRAY
                        };
                        let key_name = key
                            .map(|k| format!("{:?}", k))
                            .unwrap_or_else(|| "None".to_string());
                        let gamepad_text = gamepad
                            .map(|g| format!("GP{}", g))
                            .unwrap_or_else(|| "".to_string());
                        let binding_text = if !gamepad_text.is_empty() {
                            format!("{}: {} / {}", action_name, key_name, gamepad_text)
                        } else {
                            format!("{}: {}", action_name, key_name)
                        };
                        if i == self.controls_selection {
                            draw_text(">", screen_width() / 2.0 - 200.0, y, option_size, BLACK);
                        }
                        let binding_width =
                            measure_text(&binding_text, None, option_size as u16, 1.0).width;
                        draw_text(
                            &binding_text,
                            screen_width() / 2.0 - binding_width / 2.0,
                            y,
                            option_size,
                            color,
                        );
                    }
                    let instructions = "UP/DOWN: Select Player | LEFT/RIGHT: Select Action | ENTER: Rebind | ESC: Back";
                    let inst_width = measure_text(instructions, None, 16, 1.0).width;
                    draw_text(
                        instructions,
                        screen_width() / 2.0 - inst_width / 2.0,
                        screen_height() - 40.0,
                        16.0,
                        GRAY,
                    );
                }
            }
            GameState::Credits => {
                let title = "CREDITS";
                let title_size = MENU_TITLE_SIZE;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    60.0,
                    title_size,
                    BLACK,
                );
                let start_y = 140.0;
                let spacing = 28.0;
                let section_spacing = 40.0;
                let mut current_y = start_y;
                let game_title = "JUMP QUEST";
                let game_title_size = 36.0;
                let game_title_width =
                    measure_text(game_title, None, game_title_size as u16, 1.0).width;
                draw_text(
                    game_title,
                    screen_width() / 2.0 - game_title_width / 2.0,
                    current_y,
                    game_title_size,
                    BLACK,
                );
                current_y += section_spacing;
                let made_by = "Made by";
                let made_by_size = 20.0;
                let made_by_width = measure_text(made_by, None, made_by_size as u16, 1.0).width;
                draw_text(
                    made_by,
                    screen_width() / 2.0 - made_by_width / 2.0,
                    current_y,
                    made_by_size,
                    DARKGRAY,
                );
                current_y += spacing;
                let developer = "guicybercode";
                let developer_size = 32.0;
                let developer_width =
                    measure_text(developer, None, developer_size as u16, 1.0).width;
                draw_text(
                    developer,
                    screen_width() / 2.0 - developer_width / 2.0,
                    current_y,
                    developer_size,
                    BLACK,
                );
                current_y += section_spacing;
                let stack_title = "Technology Stack";
                let stack_title_size = 24.0;
                let stack_title_width =
                    measure_text(stack_title, None, stack_title_size as u16, 1.0).width;
                draw_text(
                    stack_title,
                    screen_width() / 2.0 - stack_title_width / 2.0,
                    current_y,
                    stack_title_size,
                    BLACK,
                );
                current_y += spacing;
                let stack_details = vec![
                    "Programming Language: Rust",
                    "Game Engine: Macroquad 0.4",
                    "Audio System: Macroquad Audio",
                    "Random Generation: rand 0.8",
                    "Rust Edition: 2021",
                ];
                let detail_size = 18.0;
                for detail in stack_details {
                    let detail_width = measure_text(detail, None, detail_size as u16, 1.0).width;
                    draw_text(
                        detail,
                        screen_width() / 2.0 - detail_width / 2.0,
                        current_y,
                        detail_size,
                        DARKGRAY,
                    );
                    current_y += spacing - 4.0;
                }
                current_y += section_spacing - spacing;
                let info_title = "Game Information";
                let info_title_size = 24.0;
                let info_title_width =
                    measure_text(info_title, None, info_title_size as u16, 1.0).width;
                draw_text(
                    info_title,
                    screen_width() / 2.0 - info_title_width / 2.0,
                    current_y,
                    info_title_size,
                    BLACK,
                );
                current_y += spacing;
                let game_info = vec![
                    format!("Version: {}", GAME_VERSION),
                    "Genre: Platformer".to_string(),
                    "Inspired by classic Jump Quest".to_string(),
                ];
                for info in game_info {
                    let info_width = measure_text(&info, None, detail_size as u16, 1.0).width;
                    draw_text(
                        &info,
                        screen_width() / 2.0 - info_width / 2.0,
                        current_y,
                        detail_size,
                        DARKGRAY,
                    );
                    current_y += spacing - 4.0;
                }
                let back_instruction = "Press ESC, ENTER or SPACE to return";
                let back_size = MENU_INSTRUCTION_SIZE;
                let back_width = measure_text(back_instruction, None, back_size as u16, 1.0).width;
                draw_text(
                    back_instruction,
                    screen_width() / 2.0 - back_width / 2.0,
                    screen_height() - 40.0,
                    back_size,
                    GRAY,
                );
            }
            GameState::Playing => {
                self.draw_level_world();
                self.draw_level_hud(false);
                if self.level_start_fade_timer > 0.0 {
                    let fade_progress = self.level_start_fade_timer / 1.5;
                    let fade_alpha = fade_progress.min(1.0);
                    draw_rectangle(
                        0.0,
                        0.0,
                        screen_width(),
                        screen_height(),
                        Color::new(0.0, 0.0, 0.0, fade_alpha),
                    );
                }
            }
            GameState::Versus => {
                let camera_x = self.camera.x;
                let camera_y = self.camera.y;
                for platform in &self.versus_platforms {
                    platform.draw(camera_x, camera_y);
                }
                if self.respawn_timer_p1 <= 0.0 {
                    self.player.draw_vs(camera_x, camera_y, true);
                }
                if let Some(ref p2) = self.player2 {
                    if self.respawn_timer_p2 <= 0.0 {
                        p2.draw_vs(camera_x, camera_y, false);
                    }
                }
                let p1_score_text = format!(
                    "P1: {} kills | {} pts",
                    self.player1_score, self.player1_points
                );
                let p2_score_text = format!(
                    "P2: {} kills | {} pts",
                    self.player2_score, self.player2_points
                );
                draw_text(&p1_score_text, 20.0, 30.0, 24.0, BLACK);
                let p2_width = measure_text(&p2_score_text, None, 24u16, 1.0).width;
                draw_text(
                    &p2_score_text,
                    screen_width() - p2_width - 20.0,
                    30.0,
                    24.0,
                    DARKGRAY,
                );
                let time_text = format!("{}", self.versus_time_remaining as u32);
                let time_width = measure_text(&time_text, None, 28u16, 1.0).width;
                let time_color = if self.versus_time_remaining < 60.0 {
                    RED
                } else if self.versus_time_remaining < 120.0 {
                    ORANGE
                } else {
                    BLACK
                };
                draw_text(
                    &time_text,
                    screen_width() / 2.0 - time_width / 2.0,
                    30.0,
                    28.0,
                    time_color,
                );
                let instructions = "P1: WASD | P2: Arrow Keys | ESC: Menu";
                let inst_width = measure_text(instructions, None, 16u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 30.0,
                    16.0,
                    GRAY,
                );
                if self.level_start_fade_timer > 0.0 {
                    let fade_progress = self.level_start_fade_timer / 1.5;
                    let fade_alpha = fade_progress.min(1.0);
                    draw_rectangle(
                        0.0,
                        0.0,
                        screen_width(),
                        screen_height(),
                        Color::new(0.0, 0.0, 0.0, fade_alpha),
                    );
                }
            }
            GameState::Coop => {
                self.draw_level_world();
                if let Some(ref p2) = self.player2 {
                    p2.draw(self.camera.x, self.camera.y);
                }
                self.draw_level_hud(true);
                let p1_text = "P1";
                let p2_text = "P2";
                draw_text(p1_text, 10.0, 10.0, 20.0, BLUE);
                if let Some(ref _p2) = self.player2 {
                    let p2_width = measure_text(p2_text, None, 20u16, 1.0).width;
                    draw_text(p2_text, screen_width() - p2_width - 10.0, 10.0, 20.0, RED);
                }
                let instructions = "P1: WASD | P2: Arrow Keys | P: Pause | ESC: Menu";
                let inst_width = measure_text(instructions, None, 16u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 30.0,
                    16.0,
                    GRAY,
                );
                if self.level_start_fade_timer > 0.0 {
                    let fade_progress = self.level_start_fade_timer / LEVEL_START_FADE_TIMER;
                    let fade_alpha = fade_progress.min(1.0);
                    draw_rectangle(
                        0.0,
                        0.0,
                        screen_width(),
                        screen_height(),
                        Color::new(0.0, 0.0, 0.0, fade_alpha),
                    );
                }
            }
            GameState::VersusEnd => {
                let title = "GAME OVER";
                let title_size = 48.0;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    150.0,
                    title_size,
                    BLACK,
                );
                let winner_text = if self.player1_points > self.player2_points {
                    "PLAYER 1 WINS!"
                } else if self.player2_points > self.player1_points {
                    "PLAYER 2 WINS!"
                } else {
                    "DRAW!"
                };
                let winner_size = 36.0;
                let winner_width = measure_text(winner_text, None, winner_size as u16, 1.0).width;
                let winner_color = if self.player1_points > self.player2_points {
                    BLACK
                } else if self.player2_points > self.player1_points {
                    DARKGRAY
                } else {
                    GRAY
                };
                draw_text(
                    winner_text,
                    screen_width() / 2.0 - winner_width / 2.0,
                    220.0,
                    winner_size,
                    winner_color,
                );
                let score_size = 28.0;
                let p1_final_text = format!(
                    "Player 1: {} kills | {} points",
                    self.player1_score, self.player1_points
                );
                let p2_final_text = format!(
                    "Player 2: {} kills | {} points",
                    self.player2_score, self.player2_points
                );
                let p1_final_width =
                    measure_text(&p1_final_text, None, score_size as u16, 1.0).width;
                let p2_final_width =
                    measure_text(&p2_final_text, None, score_size as u16, 1.0).width;
                draw_text(
                    &p1_final_text,
                    screen_width() / 2.0 - p1_final_width / 2.0,
                    300.0,
                    score_size,
                    BLACK,
                );
                draw_text(
                    &p2_final_text,
                    screen_width() / 2.0 - p2_final_width / 2.0,
                    340.0,
                    score_size,
                    DARKGRAY,
                );
                let back_text = "Press ENTER, SPACE or ESC to return to menu";
                let back_size = 18.0;
                let back_width = measure_text(back_text, None, back_size as u16, 1.0).width;
                draw_text(
                    back_text,
                    screen_width() / 2.0 - back_width / 2.0,
                    screen_height() - 50.0,
                    back_size,
                    GRAY,
                );
            }
            GameState::LevelSelect => {
                let title = "SELECT LEVEL";
                let title_size = 42.0;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    100.0,
                    title_size,
                    BLACK,
                );
                let spacing = 100.0;
                let level_count = self.unlocked_levels.len();
                let start_x = screen_width() / 2.0 - (spacing * (level_count as f32 - 1.0)) / 2.0;
                let center_y = screen_height() / 2.0;
                let level_names: Vec<String> =
                    (1..=level_count).map(|i| format!("Level {}", i)).collect();
                for i in 0..level_count {
                    let x = start_x + (i as f32 * spacing);
                    let is_selected = i == self.level_selection;
                    let is_unlocked = self.unlocked_levels[i];
                    let circle_color = if !is_unlocked {
                        DARKGRAY
                    } else if is_selected {
                        BLACK
                    } else {
                        GRAY
                    };
                    let circle_radius = if is_selected { 35.0 } else { 30.0 };
                    draw_circle(x, center_y, circle_radius, circle_color);
                    draw_circle_lines(
                        x,
                        center_y,
                        circle_radius,
                        3.0,
                        if is_selected { WHITE } else { BLACK },
                    );
                    let num_text = format!("{}", i + 1);
                    let num_size = 36.0;
                    let num_width = measure_text(&num_text, None, num_size as u16, 1.0).width;
                    draw_text(
                        &num_text,
                        x - num_width / 2.0,
                        center_y + num_size / 3.0,
                        num_size,
                        if is_unlocked {
                            if is_selected {
                                WHITE
                            } else {
                                BLACK
                            }
                        } else {
                            GRAY
                        },
                    );
                    let (difficulty, coin_count, difficulty_color) = self.get_level_info(i + 1);
                    if is_unlocked {
                        let name_size = 18.0;
                        let name_text = &level_names[i];
                        let name_width = measure_text(name_text, None, name_size as u16, 1.0).width;
                        draw_text(
                            name_text,
                            x - name_width / 2.0,
                            center_y + 60.0,
                            name_size,
                            if is_selected { BLACK } else { GRAY },
                        );
                        let diff_text = difficulty;
                        let diff_width = measure_text(&diff_text, None, 16, 1.0).width;
                        draw_text(
                            &diff_text,
                            x - diff_width / 2.0,
                            center_y + 82.0,
                            16.0,
                            difficulty_color,
                        );
                        let coins_text = format!("{} coins", coin_count);
                        let coins_width = measure_text(&coins_text, None, 14, 1.0).width;
                        draw_text(
                            &coins_text,
                            x - coins_width / 2.0,
                            center_y + 102.0,
                            14.0,
                            DARKGRAY,
                        );
                    } else {
                        let lock_text = "LOCKED";
                        let lock_size = 14.0;
                        let lock_width = measure_text(lock_text, None, lock_size as u16, 1.0).width;
                        draw_text(
                            lock_text,
                            x - lock_width / 2.0,
                            center_y + 60.0,
                            lock_size,
                            DARKGRAY,
                        );
                    }
                }
                let instructions = "ARROWS or A/D: Navigate | ENTER/SPACE: Select | ESC: Back";
                let inst_size = 18.0;
                let inst_width = measure_text(instructions, None, inst_size as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 50.0,
                    inst_size,
                    GRAY,
                );
            }
            GameState::Pause => {
                self.draw_level_world();
                self.draw_level_hud(true);
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, 0.6),
                );
                let title = "PAUSED";
                let title_size = 56.0;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    screen_height() / 2.0 - 180.0,
                    title_size,
                    WHITE,
                );
                let menu_options = vec!["RESUME", "SETTINGS", "CREDITS", "MAIN MENU"];
                let start_y = screen_height() / 2.0 - 40.0;
                for (i, option) in menu_options.iter().enumerate() {
                    let option_width =
                        measure_text(option, None, MENU_OPTION_SIZE as u16, 1.0).width;
                    let x = screen_width() / 2.0 - option_width / 2.0;
                    let y = start_y + (i as f32 * MENU_OPTION_SPACING);
                    let color = if i == self.pause_selection {
                        WHITE
                    } else {
                        LIGHTGRAY
                    };
                    if i == self.pause_selection {
                        draw_text(">", x - MENU_INDICATOR_OFFSET, y, MENU_OPTION_SIZE, WHITE);
                    }
                    draw_text(option, x, y, MENU_OPTION_SIZE, color);
                }
                let instructions = "ARROWS/WASD: Navigate | ENTER: Select | P/ESC: Resume";
                let inst_width =
                    measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 40.0,
                    MENU_INSTRUCTION_SIZE,
                    LIGHTGRAY,
                );
            }
            GameState::Respawn => {
                let time_remaining = self.respawn_timer;
                let total_time = 3.0;
                let progress = 1.0 - (time_remaining / total_time);
                let bg_alpha = (progress * 0.8).min(0.8);
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, bg_alpha),
                );
                let player_size = 64.0;
                let player_x = screen_width() / 2.0 - player_size / 2.0;
                let player_y = screen_height() / 2.0 - player_size / 2.0 - 50.0;
                let player_alpha = (1.0 - progress * 1.5).max(0.0);
                let player_color = Color::new(0.0, 0.0, 0.0, player_alpha);
                draw_rectangle(player_x, player_y, player_size, player_size, player_color);
                draw_rectangle_lines(
                    player_x,
                    player_y,
                    player_size,
                    player_size,
                    2.0,
                    Color::new(1.0, 1.0, 1.0, player_alpha),
                );
                let eye_size = 6.0;
                let eye_y = player_y + 20.0;
                draw_circle(
                    player_x + 20.0,
                    eye_y,
                    eye_size,
                    Color::new(1.0, 1.0, 1.0, player_alpha),
                );
                draw_circle(
                    player_x + 44.0,
                    eye_y,
                    eye_size,
                    Color::new(1.0, 1.0, 1.0, player_alpha),
                );
                let lives_text = format!("Lives: {}", self.lives);
                let lives_size = 48.0;
                let lives_width = measure_text(&lives_text, None, lives_size as u16, 1.0).width;
                let lives_alpha = 1.0 - (progress * 0.5).min(0.5);
                let lives_color = Color::new(1.0, 1.0, 1.0, lives_alpha);
                draw_text(
                    &lives_text,
                    screen_width() / 2.0 - lives_width / 2.0,
                    screen_height() / 2.0 + 80.0,
                    lives_size,
                    lives_color,
                );
                let countdown = (time_remaining.ceil() as u32).max(1);
                let countdown_text = format!("Respawn in {}...", countdown);
                let countdown_size = 32.0;
                let countdown_width =
                    measure_text(&countdown_text, None, countdown_size as u16, 1.0).width;
                draw_text(
                    &countdown_text,
                    screen_width() / 2.0 - countdown_width / 2.0,
                    screen_height() / 2.0 + 140.0,
                    countdown_size,
                    lives_color,
                );
            }
            GameState::GameOver => {
                let fade_progress = if self.game_over_fade_timer > 0.0 {
                    (2.0 - self.game_over_fade_timer) / 2.0
                } else {
                    1.0
                };
                let fade_alpha = fade_progress.min(1.0);
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, fade_alpha),
                );
                if fade_alpha >= 1.0 {
                    let text = "GAME OVER";
                    let text_size = 60.0;
                    let text_width = measure_text(text, None, text_size as u16, 1.0).width;
                    draw_text(
                        text,
                        screen_width() / 2.0 - text_width / 2.0,
                        screen_height() / 2.0 - 120.0,
                        text_size,
                        WHITE,
                    );
                    let score_text = format!("Score: {}", self.score);
                    let score_size = 36.0;
                    let score_width = measure_text(&score_text, None, score_size as u16, 1.0).width;
                    draw_text(
                        &score_text,
                        screen_width() / 2.0 - score_width / 2.0,
                        screen_height() / 2.0 - 40.0,
                        score_size,
                        WHITE,
                    );
                    let coins_text =
                        format!("Coins: {}/{}", self.coins_collected, self.total_coins);
                    let coins_size = 24.0;
                    let coins_width = measure_text(&coins_text, None, coins_size as u16, 1.0).width;
                    draw_text(
                        &coins_text,
                        screen_width() / 2.0 - coins_width / 2.0,
                        screen_height() / 2.0 + 10.0,
                        coins_size,
                        LIGHTGRAY,
                    );
                    let restart_text = "SPACE or ENTER: Restart | ESC: Menu";
                    let restart_size = 22.0;
                    let restart_width =
                        measure_text(restart_text, None, restart_size as u16, 1.0).width;
                    draw_text(
                        restart_text,
                        screen_width() / 2.0 - restart_width / 2.0,
                        screen_height() / 2.0 + 50.0,
                        restart_size,
                        LIGHTGRAY,
                    );
                }
            }
            GameState::LevelComplete => {
                let text = format!("LEVEL {} COMPLETE!", self.current_level);
                let text_size = 50.0;
                let text_width = measure_text(&text, None, text_size as u16, 1.0).width;
                draw_text(
                    &text,
                    screen_width() / 2.0 - text_width / 2.0,
                    screen_height() / 2.0 - 100.0,
                    text_size,
                    BLACK,
                );
                let score_text = format!("Score: {}", self.score);
                let score_size = 36.0;
                let score_width = measure_text(&score_text, None, score_size as u16, 1.0).width;
                draw_text(
                    &score_text,
                    screen_width() / 2.0 - score_width / 2.0,
                    screen_height() / 2.0 - 40.0,
                    score_size,
                    BLACK,
                );
                let coins_text = format!("Coins: {}/{}", self.coins_collected, self.total_coins);
                let coins_size = 24.0;
                let coins_width = measure_text(&coins_text, None, coins_size as u16, 1.0).width;
                draw_text(
                    &coins_text,
                    screen_width() / 2.0 - coins_width / 2.0,
                    screen_height() / 2.0 + 10.0,
                    coins_size,
                    GRAY,
                );
                let lives_text = format!("Lives: {}", self.lives);
                let lives_size = 28.0;
                let lives_width = measure_text(&lives_text, None, lives_size as u16, 1.0).width;
                draw_text(
                    &lives_text,
                    screen_width() / 2.0 - lives_width / 2.0,
                    screen_height() / 2.0 + 50.0,
                    lives_size,
                    BLACK,
                );
                if self.current_level < MAX_LEVELS
                    && self.current_level < self.unlocked_levels.len()
                    && self.unlocked_levels[self.current_level]
                {
                    let unlock_text = format!("Level {} unlocked!", self.current_level + 1);
                    let unlock_size = 24.0;
                    let unlock_width =
                        measure_text(&unlock_text, None, unlock_size as u16, 1.0).width;
                    draw_text(
                        &unlock_text,
                        screen_width() / 2.0 - unlock_width / 2.0,
                        screen_height() / 2.0 + 10.0,
                        unlock_size,
                        BLACK,
                    );
                }
                let continue_text = "ENTER/SPACE: Continue | ESC: Level Select";
                let continue_size = 20.0;
                let continue_width =
                    measure_text(continue_text, None, continue_size as u16, 1.0).width;
                draw_text(
                    continue_text,
                    screen_width() / 2.0 - continue_width / 2.0,
                    screen_height() / 2.0 + 60.0,
                    continue_size,
                    GRAY,
                );
            }
        }
        self.draw_transition();
        self.draw_error_message();
    }
    fn draw_error_message(&self) {
        if let Some(ref error) = self.error_message {
            let alpha = (self.error_timer / 5.0).min(1.0);
            let bg_color = Color::new(0.0, 0.0, 0.0, alpha * 0.8);
            let text_color = Color::new(1.0, 0.3, 0.3, alpha);
            let padding = 20.0;
            let font_size = 24.0;
            let text_width = measure_text(error, None, font_size as u16, 1.0).width;
            let box_width = text_width + padding * 2.0;
            let box_height = font_size + padding * 2.0;
            let x = screen_width() / 2.0 - box_width / 2.0;
            let y = screen_height() - box_height - 50.0;
            draw_rectangle(x, y, box_width, box_height, bg_color);
            draw_rectangle_lines(x, y, box_width, box_height, 2.0, text_color);
            draw_text(
                error,
                x + padding,
                y + padding + font_size,
                font_size,
                text_color,
            );
        }
    }
}
