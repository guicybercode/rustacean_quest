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

pub const RESOLUTIONS: [(u32, u32); 3] = [
    (800, 600),
    (1024, 768),
    (1280, 720),
];

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
