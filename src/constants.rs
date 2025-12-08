pub const GRAVITY: f32 = 800.0;
pub const ENEMY_GRAVITY: f32 = 400.0;
pub const TERMINAL_VELOCITY: f32 = 500.0;

pub const PLAYER_SPEED: f32 = 200.0;
pub const JUMP_FORCE: f32 = -400.0;
pub const PLAYER_WIDTH: f32 = 64.0;
pub const PLAYER_HEIGHT: f32 = 64.0;
pub const PLAYER_FRICTION: f32 = 0.85;

pub const ENEMY_SPEED: f32 = 50.0;
pub const ENEMY_WIDTH: f32 = 24.0;
pub const ENEMY_HEIGHT: f32 = 24.0;

pub const COIN_SIZE: f32 = 16.0;
pub const COIN_ROTATION_SPEED: f32 = 3.0;

pub const CHECKPOINT_WIDTH: f32 = 40.0;
pub const CHECKPOINT_HEIGHT: f32 = 60.0;

pub const GROUND_Y: f32 = 550.0;
pub const WORLD_WIDTH: f32 = 4200.0;
pub const LEVEL_COMPLETE_X: f32 = 4000.0;
pub const FALL_DEATH_Y: f32 = 600.0;

pub const SCREEN_WIDTH: u32 = 800;
pub const SCREEN_HEIGHT: u32 = 600;

pub const RESOLUTIONS: [(u32, u32); 3] = [(800, 600), (1024, 768), (1280, 720)];

pub const TIME_LIMIT: f32 = 300.0;
pub const TIME_WARNING_RED: f32 = 30.0;
pub const TIME_WARNING_YELLOW: f32 = 60.0;

pub const MAX_LEVELS: usize = 5;

pub const COLLISION_MARGIN: f32 = 100.0;
pub const PLATFORM_COLLISION_THRESHOLD: f32 = 20.0;

pub const SCORE_COIN: u32 = 100;
pub const SCORE_ENEMY: u32 = 200;
pub const SCORE_CHECKPOINT: u32 = 50;
pub const SCORE_LEVEL_COMPLETE: u32 = 1000;
pub const SCORE_TIME_BONUS: f32 = 10.0;

pub const MENU_TITLE_SIZE: f32 = 56.0;
pub const MENU_OPTION_SIZE: f32 = 40.0;
pub const MENU_INSTRUCTION_SIZE: f32 = 16.0;
pub const MENU_VERSION_SIZE: f32 = 14.0;
pub const MENU_OPTION_SPACING: f32 = 60.0;
pub const MENU_INDICATOR_OFFSET: f32 = 40.0;
pub const MENU_ANIMATION_SPEED: f32 = 5.0;

pub const GAME_VERSION: &str = "0.3.14";

pub const TRANSITION_DURATION: f32 = 1.0;
pub const SPLASH_DURATION: f32 = 2.0;

pub const TUTORIAL_PAGE_COUNT: usize = 5;

pub const DEFAULT_LIVES: u32 = 5;
pub const EASTER_EGG_LIVES: u32 = 15;
pub const RESPAWN_TIMER: f32 = 3.0;
pub const GAME_OVER_FADE_TIMER: f32 = 2.0;
pub const LEVEL_START_FADE_TIMER: f32 = 1.5;
pub const FOOTSTEP_INTERVAL: f32 = 0.25;
pub const MIN_VELOCITY_FOR_FOOTSTEP: f32 = 10.0;
pub const SAVE_CHECK_INTERVAL: f32 = 2.0;
pub const MAX_SAVE_SLOTS: usize = 3;
pub const MAX_NAME_LENGTH: usize = 20;
pub const MIN_NAME_LENGTH: usize = 3;
pub const ENEMY_ANIMATION_SPEED: f32 = 0.12;
pub const PLAYER_ANIMATION_SPEED: f32 = 0.08;
pub const MIN_VELOCITY_FOR_ANIMATION: f32 = 5.0;
pub const WALK_BOUNCE_SPEED: f32 = 12.0;
pub const JUMP_BOUNCE_MULTIPLIER: f32 = 0.6;
pub const ENEMY_EDGE_CHECK_OFFSET: f32 = 2.0;
pub const ENEMY_EDGE_CHECK_Y_OFFSET: f32 = 15.0;
pub const ENEMY_COLLISION_PLATFORM_OFFSET: f32 = 1.0;
pub const PLAYER_COLLISION_VELOCITY_THRESHOLD: f32 = -50.0;
pub const PLAYER_ENEMY_TOP_COLLISION_THRESHOLD: f32 = 10.0;
pub const PLAYER_ENEMY_FALLING_THRESHOLD: f32 = 0.7;
pub const ESTIMATED_ENEMIES_PER_LEVEL: usize = 20;
pub const ESTIMATED_PLATFORMS_PER_LEVEL: usize = 20;
pub const ESTIMATED_COINS_PER_LEVEL: usize = 30;
pub const ESTIMATED_CHECKPOINTS_PER_LEVEL: usize = 5;

pub const DIFFICULTY_EASY: f32 = 1.0;
pub const DIFFICULTY_NORMAL: f32 = 1.0;
pub const DIFFICULTY_HARD: f32 = 0.7;
pub const DIFFICULTY_INSANE: f32 = 0.5;

pub const CAMERA_SHAKE_INTENSITY: f32 = 5.0;
pub const CAMERA_SHAKE_DURATION: f32 = 0.2;
pub const CAMERA_SHAKE_FOOTSTEP: f32 = 1.0;
pub const CAMERA_SHAKE_JUMP: f32 = 2.0;
pub const CAMERA_SHAKE_KILL: f32 = 8.0;

pub const PARTICLE_COUNT: usize = 10;
pub const PARTICLE_LIFETIME: f32 = 0.5;
pub const PARTICLE_MIN_RADIUS: f32 = 2.0;
pub const PARTICLE_MAX_RADIUS: f32 = 5.0;
pub const COIN_BOUNCE_DURATION: f32 = 0.2;
pub const COIN_BOUNCE_SCALE: f32 = 1.3;
pub const MENU_BOUNCE_DURATION: f32 = 0.15;
pub const MENU_BOUNCE_SCALE: f32 = 1.1;

pub const FONT_SIZE_SMALL: f32 = 16.0;
pub const FONT_SIZE_MEDIUM: f32 = 24.0;
pub const FONT_SIZE_LARGE: f32 = 32.0;
pub const FONT_SIZE_EXTRA_LARGE: f32 = 48.0;

pub const ASSIST_MODE_SLOW_MOTION: f32 = 0.5;
