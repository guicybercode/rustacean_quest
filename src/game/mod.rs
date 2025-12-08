use crate::audio::AudioManager;
use crate::camera::Camera;
use crate::checkpoint::Checkpoint;
use crate::coin::Coin;
use crate::constants::*;
use crate::enemy::Enemy;
use crate::name_filter;
use crate::platform::Platform;
use crate::player::Player;
use crate::save::SaveData;
use crate::systems::{CameraShake, CoinBounce, MenuAnimation, Particle, Transition};
use macroquad::prelude::*;

mod draw;
mod helpers;
mod state;
mod update;

pub use state::*;
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
    difficulty_multiplier: f32,
    colorblind_mode: bool,
    font_size_scale: f32,
    assist_mode: bool,
    camera_shake: CameraShake,
    particles: Vec<Particle>,
    coin_bounces: Vec<CoinBounce>,
    menu_animation: MenuAnimation,
    pause_animation: MenuAnimation,
    transition: Transition,
}
impl Game {
    pub async fn new() -> Self {
        let mut unlocked_levels = vec![false; MAX_LEVELS];
        unlocked_levels[0] = true;
        let mut audio = AudioManager::new();
        audio.load_sounds().await;
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
                    let error_msg = format!("Error loading sprite P1: {:?}", e);
                    eprintln!("{}", error_msg);
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
                    let error_msg = format!("Error loading sprite P2: {:?}", e);
                    eprintln!("{}", error_msg);
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
            difficulty_multiplier: DIFFICULTY_NORMAL,
            colorblind_mode: false,
            font_size_scale: 1.0,
            assist_mode: false,
            camera_shake: CameraShake::new(),
            particles: Vec::with_capacity(PARTICLE_COUNT * 10),
            coin_bounces: Vec::new(),
            menu_animation: MenuAnimation::new(7),
            pause_animation: MenuAnimation::new(4),
            transition: Transition::new(),
        }
    }
}
