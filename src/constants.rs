// Constantes do jogo Jump Quest

// === FÍSICA ===
pub const GRAVITY: f32 = 800.0;           // Gravidade do jogador
pub const ENEMY_GRAVITY: f32 = 400.0;     // Gravidade dos inimigos
pub const TERMINAL_VELOCITY: f32 = 500.0; // Velocidade máxima de queda

// === JOGADOR ===
pub const PLAYER_SPEED: f32 = 200.0;      // Velocidade de movimento horizontal
pub const JUMP_FORCE: f32 = -400.0;       // Força do pulo (negativa = para cima)
pub const PLAYER_WIDTH: f32 = 64.0;       // Largura do jogador (aumentado para 64)
pub const PLAYER_HEIGHT: f32 = 64.0;      // Altura do jogador (aumentado para 64)
pub const PLAYER_FRICTION: f32 = 0.85;    // Atrito no chão

// === INIMIGO ===
pub const ENEMY_SPEED: f32 = 50.0;        // Velocidade dos inimigos
pub const ENEMY_WIDTH: f32 = 24.0;        // Largura do inimigo
pub const ENEMY_HEIGHT: f32 = 24.0;       // Altura do inimigo

// === MOEDA ===
pub const COIN_SIZE: f32 = 16.0;          // Tamanho da moeda
pub const COIN_ROTATION_SPEED: f32 = 3.0; // Velocidade de rotação da moeda

// === CHECKPOINT ===
pub const CHECKPOINT_WIDTH: f32 = 40.0;   // Largura do checkpoint
pub const CHECKPOINT_HEIGHT: f32 = 60.0;  // Altura do checkpoint

// === MUNDO ===
pub const GROUND_Y: f32 = 550.0;          // Posição Y do chão
pub const WORLD_WIDTH: f32 = 4200.0;      // Largura do mundo
pub const LEVEL_COMPLETE_X: f32 = 4000.0; // Posição X para completar a fase
pub const FALL_DEATH_Y: f32 = 600.0;      // Posição Y que causa morte por queda

// === TELA ===
pub const SCREEN_WIDTH: u32 = 800;        // Largura da janela
pub const SCREEN_HEIGHT: u32 = 600;       // Altura da janela

// === RESOLUÇÕES DISPONÍVEIS ===
pub const RESOLUTIONS: [(u32, u32); 3] = [
    (800, 600),
    (1024, 768),
    (1280, 720),
];

// === TEMPO ===
pub const TIME_LIMIT: f32 = 300.0;        // Tempo limite por fase (5 minutos)
pub const TIME_WARNING_RED: f32 = 30.0;   // Tempo para mostrar alerta vermelho
pub const TIME_WARNING_YELLOW: f32 = 60.0; // Tempo para mostrar alerta amarelo

// === NÍVEIS ===
pub const MAX_LEVELS: usize = 5;          // Número máximo de fases

// === COLISÃO ===
pub const COLLISION_MARGIN: f32 = 100.0;  // Margem para otimização de colisão
pub const PLATFORM_COLLISION_THRESHOLD: f32 = 20.0; // Limite para detectar colisão por cima

// === PONTUAÇÃO ===
pub const SCORE_COIN: u32 = 100;          // Pontos por moeda coletada
pub const SCORE_ENEMY: u32 = 200;         // Pontos por inimigo morto
pub const SCORE_CHECKPOINT: u32 = 50;     // Pontos por checkpoint ativado
pub const SCORE_LEVEL_COMPLETE: u32 = 1000; // Pontos por completar a fase
pub const SCORE_TIME_BONUS: f32 = 10.0;   // Pontos por segundo restante ao completar

// === MENU ===
pub const MENU_TITLE_SIZE: f32 = 56.0;     // Tamanho do título do menu
pub const MENU_OPTION_SIZE: f32 = 40.0;   // Tamanho das opções do menu
pub const MENU_INSTRUCTION_SIZE: f32 = 16.0; // Tamanho das instruções
pub const MENU_VERSION_SIZE: f32 = 14.0;  // Tamanho da versão
pub const MENU_OPTION_SPACING: f32 = 60.0; // Espaçamento entre opções
pub const MENU_INDICATOR_OFFSET: f32 = 40.0; // Offset do indicador de seleção
pub const MENU_ANIMATION_SPEED: f32 = 5.0; // Velocidade da animação de seleção

// === VERSÃO ===
pub const GAME_VERSION: &str = "0.3.14";     // Versão do jogo

// === TRANSIÇÕES ===
pub const TRANSITION_DURATION: f32 = 1.0;    // Duração da transição entre telas (em segundos)

// === TUTORIAL ===
pub const TUTORIAL_PAGE_COUNT: usize = 5;    // Número de páginas do tutorial

