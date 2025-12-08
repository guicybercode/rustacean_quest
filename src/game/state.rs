use macroquad::prelude::KeyCode;

#[derive(Clone, Copy, PartialEq)]
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
