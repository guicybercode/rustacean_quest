use macroquad::prelude::*;

use crate::player::Player;
use crate::enemy::{Enemy, create_level_enemies};
use crate::platform::{Platform, create_level_platforms};
use crate::coin::{Coin, create_level_coins};
use crate::camera::Camera;
use crate::audio::AudioManager;
use crate::checkpoint::{Checkpoint, create_level_checkpoints};
use crate::save::SaveData;
use crate::constants::*;
use crate::name_filter;

pub enum GameState {
    Menu,
    MenuExitConfirm, // Confirmação para sair
    Credits,
    Settings,
    LevelSelect,
    Playing,
    GameOver,
    LevelComplete,
    Versus, // Modo versus/coop
    VersusEnd, // Fim do modo versus (mostrar resultados)
    Respawn, // Tela de respawn após morte
    ContinueMenu, // Menu de continue para gerenciar saves
    NameInput, // Tela de entrada de nome
    Tutorial, // Tela de tutorial
    Pause, // Menu de pausa durante o jogo
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
    menu_selection: usize, // 0 = Play, 1 = Versus, 2 = Settings, 3 = Credits, 4 = Exit
    menu_animation_time: f32, // Tempo para animação de seleção
    level_selection: usize, // 0-3 para as 4 fases
    current_level: usize, // Fase atual sendo jogada
    unlocked_levels: Vec<bool>, // Fases desbloqueadas (índice 0 = fase 1, etc)
    last_checkpoint_pos: Option<(f32, f32)>, // Posição do último checkpoint ativado
    time_remaining: f32, // Tempo restante em segundos
    // Configurações
    settings_selection: usize, // 0 = Som, 1 = Resolução, 2 = Controles, 3 = Voltar
    sound_enabled: bool,
    resolution_index: usize, // Índice da resolução atual
    // Pontuação
    score: u32, // Pontuação total atual
    // Vidas
    lives: u32, // Número de vidas do jogador
    respawn_timer: f32, // Timer de respawn após morte
    game_over_fade_timer: f32, // Timer para fadeout no game over
    level_start_fade_timer: f32, // Timer para fadein ao iniciar fase
    // Áudio de passos
    footstep_timer: f32, // Timer para controlar frequência dos passos
    // Modo Versus
    player2: Option<Player>, // Segundo jogador no modo versus
    player1_score: u32, // Kills do jogador 1
    player2_score: u32, // Kills do jogador 2
    player1_streak: u32, // Contador de kills consecutivas do P1
    player2_streak: u32, // Contador de kills consecutivas do P2
    player1_points: u32, // Pontuação total do P1
    player2_points: u32, // Pontuação total do P2
    versus_platforms: Vec<Platform>, // Plataformas do mapa versus
    respawn_timer_p1: f32, // Timer de respawn do player 1
    respawn_timer_p2: f32, // Timer de respawn do player 2
    versus_time_remaining: f32, // Tempo restante no modo versus (600 segundos)
    // Sistema de saves e nome
    player_name: String, // Nome do jogador
    continue_selection: usize, // Índice do save selecionado (0-2)
    continue_mode: ContinueMode, // Modo do menu continue
    name_input: String, // Texto sendo digitado na tela de nome
    name_input_error: Option<String>, // Mensagem de erro na validação
    // Tutorial e transições
    tutorial_page: usize, // Página atual do tutorial (0-indexed)
    tutorial_completed: bool, // Se o tutorial foi completado
    transition_timer: f32, // Timer para transições entre telas
    transition_alpha: f32, // Alpha para fade in/out
    transition_target_state: Option<GameState>, // Estado de destino da transição
    versus_played: bool, // Se o modo versus já foi jogado
    has_new_save: bool, // Se há um save novo desde a última vez
    last_save_timestamp: u64, // Último timestamp de save conhecido
    save_check_timer: f32, // Timer para verificar saves novos (não a cada frame)
    level_info_cache: Vec<(String, usize, Color)>, // Cache das informações dos níveis
    // Menu de pausa
    pause_selection: usize, // Seleção no menu de pausa (0 = Resume, 1 = Settings, 2 = Credits, 3 = Main Menu)
    came_from_pause: bool, // Se veio do menu de pausa (para voltar corretamente)
    // Sprites
    player_sprite_texture_p1: Option<std::rc::Rc<Texture2D>>, // Textura do sprite do Ferris P1 (laranja)
    player_sprite_texture_p2: Option<std::rc::Rc<Texture2D>>, // Textura do sprite do Ferris P2 (escuro)
}

impl Game {
    pub async fn new() -> Self {
        // Inicialmente só a fase 1 está desbloqueada
        let mut unlocked_levels = vec![false; 4];
        unlocked_levels[0] = true; // Fase 1 desbloqueada
        
        let mut audio = AudioManager::new();
        audio.load_sounds().await;
        
        // Carregar sprites do Ferris (P1 e P2 separados)
        let player_sprite_texture_p1 = match load_texture("assets/rustcean_p1.png").await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest); // Manter pixel art nítida
                Some(std::rc::Rc::new(texture))
            },
            Err(e) => {
                eprintln!("Erro ao carregar sprite do Ferris P1: {:?}", e);
                None
            }
        };
        
        let player_sprite_texture_p2 = match load_texture("assets/rustcean_p2.png").await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest); // Manter pixel art nítida
                Some(std::rc::Rc::new(texture))
            },
            Err(e) => {
                eprintln!("Erro ao carregar sprite do Ferris P2: {:?}", e);
                None
            }
        };
        
        Self {
            player: Player::new(50.0, 400.0, player_sprite_texture_p1.as_ref().map(|t| std::rc::Rc::clone(t)), player_sprite_texture_p2.as_ref().map(|t| std::rc::Rc::clone(t))),
            enemies: Vec::new(),
            platforms: Vec::new(),
            coins: Vec::new(),
            checkpoints: Vec::new(),
            camera: Camera::new(),
            audio,
            state: GameState::Menu,
            coins_collected: 0,
            total_coins: 0,
            menu_selection: 0,
            menu_animation_time: 0.0,
            level_selection: 0,
            current_level: 1,
            unlocked_levels,
            last_checkpoint_pos: None,
            time_remaining: 0.0,
            // Configurações padrão
            settings_selection: 0,
            sound_enabled: true,
            resolution_index: 0, // 800x600 padrão
            // Pontuação
            score: 0,
            // Vidas
            lives: 5, // Começar com 5 vidas
            respawn_timer: 0.0, // Timer de respawn
            game_over_fade_timer: 0.0, // Timer de fadeout no game over
            level_start_fade_timer: 1.5, // Timer de fadein ao iniciar fase (1.5 segundos)
            // Áudio de passos
            footstep_timer: 0.0,
            // Modo Versus
            player2: None,
            player1_score: 0,
            player2_score: 0,
            player1_streak: 0,
            player2_streak: 0,
            player1_points: 0,
            player2_points: 0,
            versus_platforms: Vec::new(),
            respawn_timer_p1: 0.0,
            respawn_timer_p2: 0.0,
            versus_time_remaining: 600.0, // 10 minutos
            // Sistema de saves e nome
            player_name: String::new(),
            continue_selection: 0,
            continue_mode: ContinueMode::View,
            name_input: String::new(),
            name_input_error: None,
            // Tutorial e transições
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
            // Menu de pausa
            pause_selection: 0,
            came_from_pause: false,
            // Sprites
            player_sprite_texture_p1,
            player_sprite_texture_p2,
        }
    }
    
    /// Inicializa o cache de informações dos níveis
    fn init_level_info_cache() -> Vec<(String, usize, Color)> {
        let mut cache = Vec::new();
        for level in 1..=MAX_LEVELS {
            let coins = create_level_coins(level);
            let coin_count = coins.len();
            
            let (difficulty, color) = match level {
                1 => ("EASY".to_string(), GREEN),
                2 => ("MEDIUM".to_string(), YELLOW),
                3 => ("HARD".to_string(), Color::new(1.0, 0.65, 0.0, 1.0)), // Laranja
                4 => ("EXPERT".to_string(), RED),
                _ => ("UNKNOWN".to_string(), GRAY),
            };
            
            cache.push((difficulty, coin_count, color));
        }
        cache
    }

    fn apply_resolution(&self) {
        let (width, height) = RESOLUTIONS[self.resolution_index];
        request_new_screen_size(width as f32, height as f32);
    }

    fn save_game(&self, slot: usize) -> Result<(), String> {
        let time_taken = TIME_LIMIT - self.time_remaining;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let save_data = SaveData {
            current_level: self.current_level,
            unlocked_levels: self.unlocked_levels.clone(),
            lives: self.lives,
            score: self.score,
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
        self.last_checkpoint_pos = save_data.last_checkpoint_pos;
        self.player_name = save_data.player_name;
        self.tutorial_completed = save_data.tutorial_completed;
        self.versus_played = save_data.versus_played;
        
        Ok(())
    }

    fn has_save_file(&self) -> bool {
        // Verificar se existe pelo menos um save
        for slot in 0..3 {
            let path = SaveData::get_save_path(slot);
            if SaveData::save_exists(&path) {
                return true;
            }
        }
        false
    }

    /// Verifica se o nome do jogador é o easter egg
    fn is_easter_egg(&self) -> bool {
        self.player_name.to_lowercase() == "guicybercode"
    }

    /// Inicia uma transição para outro estado
    fn start_transition(&mut self, target_state: GameState) {
        // Mudar estado imediatamente e aplicar efeito visual
        self.state = target_state;
        self.transition_timer = 0.001; // Iniciar com valor pequeno para ativar a transição
        self.transition_alpha = 1.0; // Começar com tela preta (fade in)
        self.transition_target_state = None; // Não precisamos mais armazenar
    }

    /// Atualiza o sistema de transições
    fn update_transition(&mut self, dt: f32) {
        if self.transition_timer > 0.0 {
            self.transition_timer += dt;
            
            if self.transition_timer < TRANSITION_DURATION {
                // Fade in (tela preta vai sumindo)
                self.transition_alpha = 1.0 - (self.transition_timer / TRANSITION_DURATION).min(1.0);
            } else {
                // Transição completa
                self.transition_timer = 0.0;
                self.transition_alpha = 0.0;
            }
        }
    }

    /// Desenha overlay de transição
    fn draw_transition(&self) {
        if self.transition_timer > 0.0 && self.transition_alpha > 0.0 {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), 
                Color::new(0.0, 0.0, 0.0, self.transition_alpha));
        }
    }

    /// Verifica se há saves novos
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

    /// Retorna informações do nível (dificuldade e número de moedas) do cache
    fn get_level_info(&self, level: usize) -> (String, usize, Color) {
        if level > 0 && level <= self.level_info_cache.len() {
            self.level_info_cache[level - 1].clone()
        } else {
            ("UNKNOWN".to_string(), 0, GRAY)
        }
    }

    fn load_level(&mut self, level: usize, use_checkpoint: bool) {
        let platforms = create_level_platforms(level);
        let coins = create_level_coins(level);
        let total_coins = coins.len() as u32;
        let mut enemies = create_level_enemies(level);
        let mut checkpoints = create_level_checkpoints(level);
        
        // Se estiver usando checkpoint e houver um checkpoint salvo, restaurar estados dos checkpoints
        if use_checkpoint && self.last_checkpoint_pos.is_some() {
            // Restaurar estados dos checkpoints (marcar como ativados os que estão antes do checkpoint salvo)
            if let Some((checkpoint_x, _)) = self.last_checkpoint_pos {
                for checkpoint in &mut checkpoints {
                    if checkpoint.x <= checkpoint_x {
                        checkpoint.activated = true;
                    }
                }
            }
        }
        
        // Ajustar posições dos inimigos para ficarem sobre plataformas
        for enemy in &mut enemies {
            let enemy_center_x = enemy.x + enemy.width / 2.0;
            let mut platform_y = GROUND_Y; // Chão padrão
            let mut found = false;
            
            // Procurar a plataforma que está na mesma posição X do inimigo
            for platform in &platforms {
                // Verificar se o centro do inimigo está sobre esta plataforma (horizontalmente)
                if enemy_center_x >= platform.x 
                    && enemy_center_x <= platform.x + platform.width
                {
                    // Encontrar a plataforma mais próxima (menor Y = mais alta)
                    // que está na posição X do inimigo
                    if !found || platform.y < platform_y {
                        platform_y = platform.y;
                        found = true;
                    }
                }
            }
            
            // Ajustar Y para ficar em cima da plataforma encontrada ou no chão
            enemy.y = platform_y - enemy.height;
            enemy.on_ground = true;
            
            // Verificar imediatamente se está realmente sobre uma plataforma
            // Se não estiver, ajustar para o chão
            let mut is_on_platform = false;
            for platform in &platforms {
                if enemy.x + enemy.width / 2.0 >= platform.x 
                    && enemy.x + enemy.width / 2.0 <= platform.x + platform.width
                    && (enemy.y + enemy.height - platform.y).abs() < 2.0
                {
                    is_on_platform = true;
                    break;
                }
            }
            
            // Se não está sobre plataforma, colocar no chão
            if !is_on_platform && !found {
                enemy.y = GROUND_Y - enemy.height;
            }
        }
        
        // Posicionar player
        let (player_start_x, player_start_y) = if let (true, Some((checkpoint_x, _))) = (use_checkpoint, self.last_checkpoint_pos) {
            // Usar posição do checkpoint - jogador fica ao lado do checkpoint, no chão
            (checkpoint_x + 50.0, GROUND_Y - PLAYER_HEIGHT) // Ao lado do checkpoint, no chão
        } else {
            // Posição inicial padrão
            (50.0, GROUND_Y - PLAYER_HEIGHT)
        };
        
        let mut player = Player::new(player_start_x, player_start_y, self.player_sprite_texture_p1.as_ref().map(|t| std::rc::Rc::clone(t)), self.player_sprite_texture_p2.as_ref().map(|t| std::rc::Rc::clone(t)));
        // Garantir que o player comece no chão
        player.on_ground = true;
        player.vel_y = 0.0; // Garantir que não tem velocidade vertical inicial
        
        self.player = player;
        self.enemies = enemies;
        self.platforms = platforms;
        self.coins = coins;
        self.checkpoints = checkpoints;
        self.camera = Camera::new();
        self.coins_collected = 0;
        self.total_coins = total_coins;
        self.current_level = level;
        // Resetar tempo ao carregar nível (300 segundos = 5 minutos)
        self.time_remaining = TIME_LIMIT;
        // Resetar timer de passos
        self.footstep_timer = 0.0;
        // Iniciar fadein ao carregar level
        self.level_start_fade_timer = 1.5; // 1.5 segundos de fadein
        // Não resetar vidas ao carregar level (mantém vidas do save)
    }

    fn load_versus_map(&mut self) {
        // Criar mapa limpo para versus (preto e branco, com plataformas)
        let mut platforms = Vec::new();
        let screen_w = SCREEN_WIDTH as f32;
        
        // Chão principal
        platforms.push(Platform::new(0.0, GROUND_Y, screen_w, 50.0));
        
        // Plataformas horizontais em diferentes alturas (mapa limpo e simétrico)
        platforms.push(Platform::new(100.0, 450.0, 150.0, 20.0));
        platforms.push(Platform::new(screen_w - 250.0, 450.0, 150.0, 20.0));
        
        platforms.push(Platform::new(200.0, 350.0, 120.0, 20.0));
        platforms.push(Platform::new(screen_w - 320.0, 350.0, 120.0, 20.0));
        
        platforms.push(Platform::new(150.0, 250.0, 100.0, 20.0));
        platforms.push(Platform::new(screen_w - 250.0, 250.0, 100.0, 20.0));
        
        platforms.push(Platform::new(screen_w / 2.0 - 50.0, 400.0, 100.0, 20.0)); // Plataforma central
        
        // Plataformas verticais (canos) nas laterais
        platforms.push(Platform::new(50.0, 500.0, 40.0, 50.0));
        platforms.push(Platform::new(screen_w - 90.0, 500.0, 40.0, 50.0));
        
        self.versus_platforms = platforms;
        
        // Posicionar players em lados opostos, garantindo que estejam no chão
        // P1 à esquerda
        self.player = Player::new(100.0, GROUND_Y - PLAYER_HEIGHT, self.player_sprite_texture_p1.as_ref().map(|t| std::rc::Rc::clone(t)), self.player_sprite_texture_p2.as_ref().map(|t| std::rc::Rc::clone(t)));
        self.player.on_ground = true;
        self.player.vel_y = 0.0;
        // Garantir que está exatamente no chão
        self.player.y = GROUND_Y - PLAYER_HEIGHT;
        
        // P2 à direita
        self.player2 = Some(Player::new(screen_w - 100.0 - PLAYER_WIDTH, GROUND_Y - PLAYER_HEIGHT, self.player_sprite_texture_p1.as_ref().map(|t| std::rc::Rc::clone(t)), self.player_sprite_texture_p2.as_ref().map(|t| std::rc::Rc::clone(t))));
        if let Some(ref mut p2) = self.player2 {
            p2.on_ground = true;
            p2.vel_y = 0.0;
            // Garantir que está exatamente no chão (mesma altura do P1)
            p2.y = GROUND_Y - PLAYER_HEIGHT;
        }
        
        // Resetar pontuações, streaks e timers
        self.player1_score = 0;
        self.player2_score = 0;
        self.player1_streak = 0;
        self.player2_streak = 0;
        self.player1_points = 0;
        self.player2_points = 0;
        self.respawn_timer_p1 = 0.0;
        self.respawn_timer_p2 = 0.0;
        self.versus_time_remaining = 600.0; // 10 minutos
        self.camera = Camera::new();
    }

    pub fn update(&mut self, dt: f32) {
        // Atualizar sistema de transições
        self.update_transition(dt);
        
        // Verificar saves novos no menu (apenas a cada 2 segundos, não a cada frame)
        if matches!(self.state, GameState::Menu) {
            self.save_check_timer += dt;
            if self.save_check_timer >= 2.0 {
                self.check_for_new_saves();
                self.save_check_timer = 0.0;
            }
        } else {
            self.save_check_timer = 0.0;
        }
        
        match self.state {
            GameState::Menu => {
                // Atualizar animação de seleção
                self.menu_animation_time += dt * MENU_ANIMATION_SPEED;
                
                // Navegação no menu
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if self.menu_selection > 0 {
                        self.menu_selection -= 1;
                        self.menu_animation_time = 0.0; // Resetar animação ao mudar seleção
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if self.menu_selection < 5 { // 6 opções: Continue, Play, Versus, Settings, Credits, Exit
                        self.menu_selection += 1;
                        self.menu_animation_time = 0.0; // Resetar animação ao mudar seleção
                        self.audio.play_menu_select();
                    }
                }
                
                // Selecionar opção
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.audio.play_menu_select();
                    
                    match self.menu_selection {
                        0 => {
                            // Continue - ir para menu de continue
                            self.has_new_save = false; // Resetar indicador ao entrar
                            self.start_transition(GameState::ContinueMenu);
                            self.continue_selection = 0;
                            self.continue_mode = ContinueMode::View;
                        }
                        1 => {
                            // Play - ir para entrada de nome
                            self.start_transition(GameState::NameInput);
                            self.name_input.clear();
                            self.name_input_error = None;
                        }
                        2 => {
                            // Versus - iniciar modo versus
                            self.versus_played = true;
                            self.load_versus_map();
                            self.start_transition(GameState::Versus);
                        }
                        3 => {
                            // Configurações
                            self.state = GameState::Settings;
                            self.settings_selection = 0;
                        }
                        4 => {
                            // Créditos
                            self.state = GameState::Credits;
                        }
                        5 => {
                            // Sair - pedir confirmação
                            self.state = GameState::MenuExitConfirm;
                            self.menu_selection = 0; // Resetar seleção para "SIM"
                        }
                        _ => {}
                    }
                }
            }
            GameState::MenuExitConfirm => {
                // Confirmação para sair
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    // Confirmar saída
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace) {
                    // Cancelar - voltar ao menu
                    self.audio.play_menu_select();
                    self.state = GameState::Menu;
                }
            }
            GameState::NameInput => {
                // Capturar caracteres digitados
                if let Some(ch) = get_char_pressed() {
                    if ch.is_alphanumeric() || ch == ' ' || ch == '-' || ch == '_' {
                        if self.name_input.len() < 20 {
                            self.name_input.push(ch);
                            self.name_input_error = None;
                        }
                    }
                }
                
                // Backspace para apagar
                if is_key_pressed(KeyCode::Backspace) {
                    self.name_input.pop();
                    self.name_input_error = None;
                }
                
                // Validar nome em tempo real
                let (is_valid, error_msg) = name_filter::is_name_valid(&self.name_input);
                if !is_valid && !self.name_input.is_empty() {
                    self.name_input_error = error_msg;
                } else {
                    self.name_input_error = None;
                }
                
                // ENTER para confirmar (se válido)
                if is_key_pressed(KeyCode::Enter) {
                    let (is_valid, _) = name_filter::is_name_valid(&self.name_input);
                    if is_valid {
                        self.player_name = self.name_input.clone();
                        // Easter egg: se o nome for "guicybercode", começar com 15 vidas
                        if self.player_name.to_lowercase() == "guicybercode" {
                            self.lives = 15;
                        }
                        self.state = GameState::LevelSelect;
                        self.level_selection = 0;
                    }
                }
                
                // ESC para cancelar
                if is_key_pressed(KeyCode::Escape) {
                    self.name_input.clear();
                    self.name_input_error = None;
                    self.state = GameState::Menu;
                }
            }
            GameState::ContinueMenu => {
                if self.continue_mode == ContinueMode::View {
                    // Navegação entre slots
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
                    
                    // ENTER para carregar save
                    if is_key_pressed(KeyCode::Enter) {
                        // Verificar apenas quando necessário (não a cada frame)
                        let path = SaveData::get_save_path(self.continue_selection);
                        if SaveData::save_exists(&path) {
                            if let Err(e) = self.load_game(self.continue_selection) {
                                eprintln!("Erro ao carregar save: {}", e);
                            } else {
                                self.load_level(self.current_level, self.last_checkpoint_pos.is_some());
                                self.start_transition(GameState::Playing);
                            }
                        }
                    }
                    
                    // DELETE/BACKSPACE para apagar save
                    if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
                        let path = SaveData::get_save_path(self.continue_selection);
                        if SaveData::save_exists(&path) {
                            self.continue_mode = ContinueMode::DeleteConfirm;
                        }
                    }
                    
                    // ESC para voltar ao menu
                    if is_key_pressed(KeyCode::Escape) {
                        self.state = GameState::Menu;
                    }
                } else {
                    // Modo DeleteConfirm
                    if is_key_pressed(KeyCode::Y) {
                        // Confirmar apagar
                        if let Err(e) = SaveData::delete_save(self.continue_selection) {
                            eprintln!("Erro ao apagar save: {}", e);
                        }
                        self.continue_mode = ContinueMode::View;
                    }
                    if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
                        // Cancelar
                        self.continue_mode = ContinueMode::View;
                    }
                }
            }
            GameState::Settings => {
                // Navegação nas configurações
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if self.settings_selection > 0 {
                        self.settings_selection -= 1;
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if self.settings_selection < 3 { // 4 opções: Som, Resolução, Controles, Voltar
                        self.settings_selection += 1;
                        self.audio.play_menu_select();
                    }
                }
                
                // Ajustar valores com esquerda/direita
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    match self.settings_selection {
                        0 => {
                            // Toggle som
                            self.sound_enabled = !self.sound_enabled;
                            self.audio.set_enabled(self.sound_enabled);
                            self.audio.play_menu_select();
                        }
                        1 => {
                            // Previous resolution
                            if self.resolution_index > 0 {
                                self.resolution_index -= 1;
                                self.apply_resolution();
                                self.audio.play_menu_select();
                            }
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    match self.settings_selection {
                        0 => {
                            // Toggle som
                            self.sound_enabled = !self.sound_enabled;
                            self.audio.set_enabled(self.sound_enabled);
                            self.audio.play_menu_select();
                        }
                        1 => {
                            // Next resolution
                            if self.resolution_index < RESOLUTIONS.len() - 1 {
                                self.resolution_index += 1;
                                self.apply_resolution();
                                self.audio.play_menu_select();
                            }
                        }
                        _ => {}
                    }
                }
                
                // Selecionar opção
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    match self.settings_selection {
                        0 => {
                            // Toggle som
                            self.sound_enabled = !self.sound_enabled;
                            self.audio.set_enabled(self.sound_enabled);
                            self.audio.play_menu_select();
                        }
                        3 => {
                            // Voltar
                            self.audio.play_menu_select();
                            if self.came_from_pause {
                                // Voltar para o jogo pausado
                                self.came_from_pause = false;
                                self.state = GameState::Pause;
                            } else {
                                // Voltar para o menu principal
                                self.state = GameState::Menu;
                                self.menu_selection = 0;
                            }
                        }
                        _ => {}
                    }
                }
                
                // ESC para voltar
                if is_key_pressed(KeyCode::Escape) {
                    self.audio.play_menu_select();
                    if self.came_from_pause {
                        // Voltar para o jogo pausado
                        self.came_from_pause = false;
                        self.state = GameState::Pause;
                    } else {
                        // Voltar para o menu principal
                        self.state = GameState::Menu;
                        self.menu_selection = 0;
                    }
                }
            }
            GameState::Credits => {
                // Voltar ao menu ou ao jogo pausado
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    if self.came_from_pause {
                        // Voltar para o jogo pausado
                        self.came_from_pause = false;
                        self.state = GameState::Pause;
                    } else {
                        // Voltar para o menu principal
                        self.state = GameState::Menu;
                        self.menu_selection = 0;
                    }
                }
            }
            GameState::Tutorial => {
                // Navegação entre páginas
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
                
                // ENTER/SPACE para pular tutorial ou avançar
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    if self.tutorial_page == TUTORIAL_PAGE_COUNT - 1 {
                        // Última página - completar tutorial
                        self.tutorial_completed = true;
                        self.start_transition(GameState::LevelSelect);
                    } else {
                        // Avançar para próxima página
                        if self.tutorial_page < TUTORIAL_PAGE_COUNT - 1 {
                            self.tutorial_page += 1;
                        }
                    }
                }
                
                // ESC para voltar ao menu ou pular tutorial
                if is_key_pressed(KeyCode::Escape) {
                    self.start_transition(GameState::LevelSelect);
                }
            }
            GameState::LevelSelect => {
                // Navegação no seletor de fases (pula fases bloqueadas)
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    // Encontrar próxima fase desbloqueada à esquerda
                    for i in (0..self.level_selection).rev() {
                        if self.unlocked_levels[i] {
                            self.level_selection = i;
                            break;
                        }
                    }
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    // Encontrar próxima fase desbloqueada à direita
                    for i in (self.level_selection + 1)..4 {
                        if self.unlocked_levels[i] {
                            self.level_selection = i;
                            break;
                        }
                    }
                }
                
                // Selecionar fase (apenas se desbloqueada)
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    if self.level_selection < self.unlocked_levels.len() 
                        && self.unlocked_levels[self.level_selection] 
                    {
                        // Se for nível 1 e tutorial não foi completado, oferecer tutorial
                        if self.level_selection == 0 && !self.tutorial_completed {
                            self.start_transition(GameState::Tutorial);
                        } else {
                            self.last_checkpoint_pos = None; // Resetar checkpoint ao iniciar novo nível
                            self.load_level(self.level_selection + 1, false);
                            self.score = 0; // Resetar pontuação ao iniciar novo nível
                            self.start_transition(GameState::Playing);
                        }
                    }
                }
                
                // Voltar ao menu
                if is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Menu;
                    self.menu_selection = 0;
                }
            }
            GameState::Playing => {
                // Verificar se quer pausar (P)
                if is_key_pressed(KeyCode::P) {
                    self.state = GameState::Pause;
                    self.pause_selection = 0;
                    self.came_from_pause = false; // Resetar flag
                    return; // Não atualizar o jogo quando pausar
                }
                
                // Verificar se quer voltar ao menu (ESC)
                if is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Menu;
                    self.menu_selection = 0;
                    return;
                }
                
                // Atualizar fadein ao iniciar fase
                if self.level_start_fade_timer > 0.0 {
                    self.level_start_fade_timer -= dt;
                }
                
                // Atualizar contador de tempo
                self.time_remaining -= dt;
                
                // Verificar se o tempo acabou
                if self.time_remaining <= 0.0 {
                    self.time_remaining = 0.0;
                    self.audio.play_death();
                    self.state = GameState::GameOver;
                    return; // Sair do update para não processar mais nada
                }
                
                // Processar movimento horizontal primeiro
                self.player.handle_movement();
                
                // Atualizar física do jogador
                self.player.update(dt);
                
                // Cache do retângulo do jogador (evitar múltiplas chamadas)
                let (px, py, pw, ph) = self.player.get_rect();
                
                // Verificar checkpoints
                for checkpoint in &mut self.checkpoints {
                    if checkpoint.check_activation(px, py, pw, ph) {
                        // Novo checkpoint ativado - salvar posição
                        self.last_checkpoint_pos = Some((checkpoint.x, checkpoint.y));
                        self.score += SCORE_CHECKPOINT; // Adicionar pontos
                        self.audio.play_coin(); // Usar som de moeda como feedback
                    }
                }
                
                // Verificar colisões jogador-plataforma (ANTES do pulo para garantir on_ground correto)
                for platform in &self.platforms {
                    // Verificar apenas se a plataforma está na tela ou próxima do jogador
                    if platform.x + platform.width >= px - COLLISION_MARGIN 
                        && platform.x <= px + pw + COLLISION_MARGIN
                        && platform.y + platform.height >= py - COLLISION_MARGIN
                        && platform.y <= py + ph + COLLISION_MARGIN
                    {
                        self.player.check_platform_collision(platform);
                    }
                }
                
                // Processar pulo DEPOIS das colisões (para usar on_ground atualizado)
                let jumped = self.player.handle_jump();
                
                // Tocar som de pulo
                if jumped {
                    self.audio.play_jump(self.is_easter_egg());
                }
                
                // Atualizar animação DEPOIS das colisões (para usar on_ground correto)
                self.player.update_animation(dt);
                
                // Tocar som de passos quando estiver andando no chão
                const FOOTSTEP_INTERVAL: f32 = 0.25; // Intervalo entre passos (em segundos)
                if self.player.on_ground && self.player.vel_x.abs() > 10.0 {
                    // Personagem está no chão e se movendo
                    self.footstep_timer += dt;
                    if self.footstep_timer >= FOOTSTEP_INTERVAL {
                        self.audio.play_footstep(self.is_easter_egg());
                        self.footstep_timer = 0.0; // Resetar timer
                    }
                } else {
                    // Resetar timer quando não está andando
                    self.footstep_timer = 0.0;
                }
                
                // Verificar colisão com paredes invisíveis (bordas laterais do mundo)
                let player_left = self.player.x;
                let player_right = self.player.x + self.player.width;
                
                // Parede esquerda do mundo
                if player_left < 0.0 {
                    self.player.x = 0.0;
                    self.player.vel_x = 0.0;
                }
                // Parede direita do mundo
                if player_right > WORLD_WIDTH {
                    self.player.x = WORLD_WIDTH - self.player.width;
                    self.player.vel_x = 0.0;
                }
                
                // Verificar se caiu do mapa (morte por queda)
                if self.player.y > FALL_DEATH_Y {
                    self.audio.play_death();
                    // Perder uma vida
                    if self.lives > 0 {
                        self.lives -= 1;
                    }
                    // Se não tem mais vidas, game over com fadeout
                    if self.lives == 0 {
                        self.game_over_fade_timer = 2.0; // 2 segundos de fadeout
                        self.state = GameState::GameOver;
                    } else {
                        // Iniciar tela de respawn
                        self.respawn_timer = 3.0; // 3 segundos de respawn
                        self.state = GameState::Respawn;
                    }
                }
                
                // Atualizar inimigos
                for enemy in &mut self.enemies {
                    if !enemy.alive {
                        continue; // Pular inimigos mortos
                    }
                    
                    enemy.update(dt);
                    
                    // Colisão inimigo-plataforma
                    for platform in &self.platforms {
                        // Verificar apenas se a plataforma está próxima do inimigo
                        if platform.x + platform.width >= enemy.x - COLLISION_MARGIN 
                            && platform.x <= enemy.x + enemy.width + COLLISION_MARGIN
                            && platform.y + platform.height >= enemy.y - COLLISION_MARGIN
                            && platform.y <= enemy.y + enemy.height + COLLISION_MARGIN
                        {
                            enemy.check_platform_collision(platform);
                        }
                    }
                    
                    // Verificar bordas para não cair das plataformas (só se estiver no chão)
                    if enemy.on_ground {
                        enemy.check_edge(&self.platforms);
                    }
                    
                    // Colisão inimigo-chão
                    enemy.check_ground_collision(GROUND_Y);
                    
                    // Colisão jogador-inimigo (usar cache)
                    match enemy.check_player_collision(px, py, pw, ph, self.player.vel_y) {
                        Some(true) => {
                            // Jogador morreu
                            self.audio.play_death();
                            // Perder uma vida
                            if self.lives > 0 {
                                self.lives -= 1;
                            }
                            // Se não tem mais vidas, game over com fadeout
                            if self.lives == 0 {
                                self.game_over_fade_timer = 2.0; // 2 segundos de fadeout
                                self.state = GameState::GameOver;
                            } else {
                                // Iniciar tela de respawn
                                self.respawn_timer = 3.0; // 3 segundos de respawn
                                self.state = GameState::Respawn;
                            }
                            break; // Sair do loop se morreu
                        }
                        Some(false) => {
                            // Inimigo morreu - jogador quica
                            self.audio.play_enemy_death();
                            self.score += SCORE_ENEMY; // Adicionar pontos por matar inimigo
                            self.player.vel_y = JUMP_FORCE * 0.6; // Quique menor que pulo normal
                        }
                        None => {
                            // Sem colisão
                        }
                    }
                }
                
                // Atualizar moedas (otimizado: só verificar moedas não coletadas)
                for coin in &mut self.coins {
                    if coin.collected {
                        continue; // Pular moedas já coletadas
                    }
                    
                    coin.update(dt);
                    if coin.check_collection(px, py, pw, ph) {
                        self.coins_collected += 1;
                        self.score += SCORE_COIN; // Adicionar pontos por moeda
                        self.audio.play_coin();
                    }
                }
                
                // Verificar se completou a fase (chegou ao final ou coletou todas as moedas)
                if self.player.x > LEVEL_COMPLETE_X || self.coins_collected >= self.total_coins {
                    // Calcular bônus de tempo (pontos por segundo restante)
                    let time_bonus = (self.time_remaining * SCORE_TIME_BONUS) as u32;
                    self.score += SCORE_LEVEL_COMPLETE + time_bonus; // Pontos por completar + bônus de tempo
                    
                    self.audio.play_level_complete();
                    // Desbloquear próxima fase se existir
                    if self.current_level < 4 && self.current_level < self.unlocked_levels.len() {
                        self.unlocked_levels[self.current_level] = true; // Desbloquear próxima fase
                    }
                    // Salvar progresso após completar level (usar slot 0 por padrão)
                    if let Err(e) = self.save_game(0) {
                        eprintln!("Erro ao salvar jogo: {}", e);
                    }
                    self.state = GameState::LevelComplete;
                }
                
                // Atualizar câmera
                let screen_width = screen_width();
                self.camera.update(self.player.x, screen_width);
            }
            GameState::LevelComplete => {
                // Verificar se quer continuar
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.state = GameState::LevelSelect;
                    // Selecionar próxima fase se desbloqueada, senão volta para a atual
                    // current_level é 1-indexed, unlocked_levels é 0-indexed
                    if self.current_level < 4 
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
                // Verificar se quer voltar ao menu
                if is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Menu;
                    self.menu_selection = 0;
                    self.player2 = None;
                    return;
                }
                
                // Atualizar timer
                self.versus_time_remaining -= dt;
                
                // Verificar se o tempo acabou
                if self.versus_time_remaining <= 0.0 {
                    self.versus_time_remaining = 0.0;
                    self.audio.play_level_complete(); // Som de fim de jogo
                    self.state = GameState::VersusEnd;
                    return;
                }
                
                // Processar respawns
                if self.respawn_timer_p1 > 0.0 {
                    self.respawn_timer_p1 -= dt;
                    if self.respawn_timer_p1 <= 0.0 {
                        // Respawnar player 1
                        self.player = Player::new(100.0, GROUND_Y - PLAYER_HEIGHT, self.player_sprite_texture_p1.as_ref().map(|t| std::rc::Rc::clone(t)), self.player_sprite_texture_p2.as_ref().map(|t| std::rc::Rc::clone(t)));
                        self.player.on_ground = true;
                        self.player.vel_y = 0.0;
                        // Garantir que está exatamente no chão
                        self.player.y = GROUND_Y - PLAYER_HEIGHT;
                    }
                }
                
                if self.respawn_timer_p2 > 0.0 {
                    self.respawn_timer_p2 -= dt;
                    if let Some(ref mut p2) = self.player2 {
                        if self.respawn_timer_p2 <= 0.0 {
                            // Respawnar player 2
                            *p2 = Player::new(700.0, GROUND_Y - PLAYER_HEIGHT, self.player_sprite_texture_p1.as_ref().map(|t| std::rc::Rc::clone(t)), self.player_sprite_texture_p2.as_ref().map(|t| std::rc::Rc::clone(t)));
                            p2.on_ground = true;
                            p2.vel_y = 0.0;
                            // Garantir que está exatamente no chão (mesma altura do P1)
                            p2.y = GROUND_Y - PLAYER_HEIGHT;
                        }
                    }
                }
                
                // Atualizar player 1 (WASD) - apenas se não estiver em respawn
                if self.respawn_timer_p1 <= 0.0 {
                    // Movimento P1 (WASD)
                    let p1_left = is_key_down(KeyCode::A);
                    let p1_right = is_key_down(KeyCode::D);
                    self.player.handle_movement_custom(p1_left, p1_right);
                    
                    self.player.update(dt);
                    
                    // Colisões P1 com plataformas
                    let (px, py, pw, ph) = self.player.get_rect();
                    for platform in &self.versus_platforms {
                        if platform.check_collision(px, py, pw, ph) {
                            self.player.check_platform_collision(platform);
                        }
                    }
                    
                    // Garantir que P1 está no chão se não estiver em plataforma
                    if self.player.on_ground && self.player.vel_y == 0.0 {
                        // Verificar se está realmente sobre uma plataforma
                        let mut is_on_platform = false;
                        for platform in &self.versus_platforms {
                            if px + pw / 2.0 >= platform.x 
                                && px + pw / 2.0 <= platform.x + platform.width
                                && (self.player.y + PLAYER_HEIGHT - platform.y).abs() < 5.0
                            {
                                is_on_platform = true;
                                break;
                            }
                        }
                        // Se não está sobre plataforma, colocar no chão
                        if !is_on_platform {
                            self.player.y = GROUND_Y - PLAYER_HEIGHT;
                        }
                    }
                    
                    // Atualizar animação P1 (depois das colisões)
                    self.player.update_animation(dt);
                    
                    // Pulo P1 (W ou Space) - usar is_key_down para pulos mais rápidos
                    let p1_jump = is_key_down(KeyCode::W) || is_key_down(KeyCode::Space);
                    let jumped = self.player.handle_jump_custom(p1_jump);
                    if jumped {
                        self.audio.play_jump(self.is_easter_egg());
                    }
                    
                    // Som de passos P1
                    const FOOTSTEP_INTERVAL: f32 = 0.25;
                    if self.player.on_ground && self.player.vel_x.abs() > 10.0 {
                        self.footstep_timer += dt;
                        if self.footstep_timer >= FOOTSTEP_INTERVAL {
                            self.audio.play_footstep(self.is_easter_egg());
                            self.footstep_timer = 0.0;
                        }
                    } else {
                        self.footstep_timer = 0.0;
                    }
                }
                
                // Atualizar player 2 (Setas) - apenas se não estiver em respawn
                if let Some(ref mut p2) = self.player2 {
                    if self.respawn_timer_p2 <= 0.0 {
                        // Movimento P2 (Setas)
                        let p2_left = is_key_down(KeyCode::Left);
                        let p2_right = is_key_down(KeyCode::Right);
                        p2.handle_movement_custom(p2_left, p2_right);
                        
                        p2.update(dt);
                        
                        // Colisões P2 com plataformas
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        for platform in &self.versus_platforms {
                            if platform.check_collision(px2, py2, pw2, ph2) {
                                p2.check_platform_collision(platform);
                            }
                        }
                        
                        // Garantir que P2 está no chão se não estiver em plataforma (mesma altura do P1)
                        if p2.on_ground && p2.vel_y == 0.0 {
                            // Verificar se está realmente sobre uma plataforma
                            let mut is_on_platform = false;
                            let mut platform_y = GROUND_Y;
                            for platform in &self.versus_platforms {
                                if px2 + pw2 / 2.0 >= platform.x 
                                    && px2 + pw2 / 2.0 <= platform.x + platform.width
                                    && (p2.y + PLAYER_HEIGHT - platform.y).abs() < 5.0
                                {
                                    is_on_platform = true;
                                    platform_y = platform.y;
                                    break;
                                }
                            }
                            // Se não está sobre plataforma, colocar no chão (mesma altura do P1)
                            if !is_on_platform {
                                p2.y = GROUND_Y - PLAYER_HEIGHT;
                            } else {
                                // Se está sobre plataforma, garantir que está exatamente em cima
                                p2.y = platform_y - PLAYER_HEIGHT;
                            }
                        }
                        
                        // Atualizar animação P2 (depois das colisões)
                        p2.update_animation(dt);
                        
                        // Pulo P2 (Seta para cima) - usar is_key_down para pulos mais rápidos
                        let p2_jump = is_key_down(KeyCode::Up);
                        let jumped = p2.handle_jump_custom(p2_jump);
                        if jumped {
                            // No modo versus, não há easter egg (só no single player)
                            self.audio.play_jump(false);
                        }
                    } else {
                        // Durante respawn, ainda atualizar física (mas não controles)
                        p2.update(dt);
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        for platform in &self.versus_platforms {
                            if platform.check_collision(px2, py2, pw2, ph2) {
                                p2.check_platform_collision(platform);
                            }
                        }
                    }
                }
                
                // Atualizar player 1 durante respawn também
                if self.respawn_timer_p1 > 0.0 {
                    self.player.update(dt);
                    let (px, py, pw, ph) = self.player.get_rect();
                    for platform in &self.versus_platforms {
                        if platform.check_collision(px, py, pw, ph) {
                            self.player.check_platform_collision(platform);
                        }
                    }
                }
                
                // Verificar colisões entre players (morte por pulo)
                if self.respawn_timer_p1 <= 0.0 && self.respawn_timer_p2 <= 0.0 {
                    if let Some(ref mut p2) = self.player2 {
                        // P1 pulou em cima de P2
                        if self.player.check_stomp(p2, self.player.vel_y) {
                            // Incrementar kills
                            self.player1_score += 1;
                            // Incrementar streak e resetar streak do P2
                            self.player1_streak += 1;
                            self.player2_streak = 0;
                            // Calcular pontos: 200 * (2 ^ (streak - 1))
                            let points = if self.player1_streak > 0 {
                                200 * (1 << (self.player1_streak - 1))
                            } else {
                                200
                            };
                            self.player1_points += points;
                            
                            self.audio.play_enemy_death(); // Som de eliminação
                            self.respawn_timer_p2 = 2.0; // 2 segundos de respawn
                            // Dar um pequeno impulso para P1
                            self.player.vel_y = JUMP_FORCE * 0.6;
                        }
                        // P2 pulou em cima de P1
                        else if p2.check_stomp(&self.player, p2.vel_y) {
                            // Incrementar kills
                            self.player2_score += 1;
                            // Incrementar streak e resetar streak do P1
                            self.player2_streak += 1;
                            self.player1_streak = 0;
                            // Calcular pontos: 200 * (2 ^ (streak - 1))
                            let points = if self.player2_streak > 0 {
                                200 * (1 << (self.player2_streak - 1))
                            } else {
                                200
                            };
                            self.player2_points += points;
                            
                            self.audio.play_enemy_death(); // Som de eliminação
                            self.respawn_timer_p1 = 2.0; // 2 segundos de respawn
                            // Dar um pequeno impulso para P2
                            p2.vel_y = JUMP_FORCE * 0.6;
                        }
                    }
                }
                
                // Verificar colisão com paredes invisíveis (bordas laterais) - P1
                // Verificar colisão com paredes invisíveis (bordas laterais do mundo) - P1
                let p1_left = self.player.x;
                let p1_right = self.player.x + self.player.width;
                
                // Parede esquerda do mundo
                if p1_left < 0.0 {
                    self.player.x = 0.0;
                    self.player.vel_x = 0.0;
                }
                // Parede direita do mundo
                if p1_right > WORLD_WIDTH {
                    self.player.x = WORLD_WIDTH - self.player.width;
                    self.player.vel_x = 0.0;
                }
                
                // Verificar colisão com paredes invisíveis - P2
                if let Some(ref mut p2) = self.player2 {
                    let p2_left = p2.x;
                    let p2_right = p2.x + p2.width;
                    
                    // Parede esquerda do mundo
                    if p2_left < 0.0 {
                        p2.x = 0.0;
                        p2.vel_x = 0.0;
                    }
                    // Parede direita do mundo
                    if p2_right > WORLD_WIDTH {
                        p2.x = WORLD_WIDTH - p2.width;
                        p2.vel_x = 0.0;
                    }
                }
                
                // Verificar se caiu do mapa (morte por queda)
                if self.player.y > FALL_DEATH_Y && self.respawn_timer_p1 <= 0.0 {
                    // P1 caiu - P2 ganha pontos
                    self.player2_score += 1;
                    self.player2_streak += 1;
                    self.player1_streak = 0; // Resetar streak do P1
                    // Calcular pontos: 200 * (2 ^ (streak - 1))
                    let points = if self.player2_streak > 0 {
                        200 * (1 << (self.player2_streak - 1))
                    } else {
                        200
                    };
                    self.player2_points += points;
                    
                    self.audio.play_enemy_death(); // Som de eliminação
                    self.respawn_timer_p1 = 2.0;
                }
                
                if let Some(ref p2) = self.player2 {
                    if p2.y > FALL_DEATH_Y && self.respawn_timer_p2 <= 0.0 {
                        // P2 caiu - P1 ganha pontos
                        self.player1_score += 1;
                        self.player1_streak += 1;
                        self.player2_streak = 0; // Resetar streak do P2
                        // Calcular pontos: 200 * (2 ^ (streak - 1))
                        let points = if self.player1_streak > 0 {
                            200 * (1 << (self.player1_streak - 1))
                        } else {
                            200
                        };
                        self.player1_points += points;
                        
                        self.audio.play_enemy_death(); // Som de eliminação
                        self.respawn_timer_p2 = 2.0;
                    }
                }
                
                // Atualizar câmera para focar no centro entre os dois players
                if let Some(ref p2) = self.player2 {
                    let center_x = (self.player.x + p2.x) / 2.0;
                    let screen_width = screen_width();
                    self.camera.update(center_x, screen_width);
                }
            }
            GameState::VersusEnd => {
                // Tela de resultados finais
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Menu;
                    self.menu_selection = 0;
                    self.player2 = None;
                }
            }
            GameState::Pause => {
                // Navegação no menu de pausa
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
                
                // Selecionar opção
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.audio.play_menu_select();
                    match self.pause_selection {
                        0 => {
                            // Resume - voltar ao jogo
                            self.state = GameState::Playing;
                        }
                        1 => {
                            // Settings - ir para configurações
                            self.came_from_pause = true;
                            self.state = GameState::Settings;
                            self.settings_selection = 0;
                        }
                        2 => {
                            // Credits - ir para créditos
                            self.came_from_pause = true;
                            self.state = GameState::Credits;
                        }
                        3 => {
                            // Main Menu - voltar ao menu principal
                            self.state = GameState::Menu;
                            self.menu_selection = 0;
                        }
                        _ => {}
                    }
                }
                
                // P ou ESC para resumir
                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Playing;
                }
            }
            GameState::Respawn => {
                // Atualizar timer de respawn
                self.respawn_timer -= dt;
                
                // Quando o timer acabar, renascer
                if self.respawn_timer <= 0.0 {
                    self.respawn_timer = 0.0;
                    // Renascer no último checkpoint
                    self.load_level(self.current_level, self.last_checkpoint_pos.is_some());
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                // Atualizar timer de fadeout (se ainda estiver fazendo fade)
                if self.game_over_fade_timer > 0.0 {
                    self.game_over_fade_timer -= dt;
                }
                
                // Só permitir interação após o fadeout terminar
                if self.game_over_fade_timer <= 0.0 {
                    // Verificar se quer reiniciar ou voltar ao menu
                    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                        // Resetar vidas para 5 ao reiniciar
                        self.lives = 5;
                        // Renascer no último checkpoint se houver, senão no início
                        self.load_level(self.current_level, self.last_checkpoint_pos.is_some());
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

    pub fn draw(&self) {
        clear_background(WHITE);
        
        match self.state {
            GameState::Menu => {
                // Título do jogo (maior e mais destacado)
                let title = "JUMP QUEST";
                let title_width = measure_text(title, None, MENU_TITLE_SIZE as u16, 1.0).width;
                let title_color = if self.is_easter_egg() {
                    // Cor dourada para easter egg com brilho
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
                
                // Menu options with better spacing
                let menu_options = vec!["CONTINUE", "PLAY", "VERSUS", "SETTINGS", "CREDITS", "EXIT"];
                let start_y = screen_height() / 2.0 - 40.0;
                
                for (i, option) in menu_options.iter().enumerate() {
                    let option_width = measure_text(option, None, MENU_OPTION_SIZE as u16, 1.0).width;
                    let x = screen_width() / 2.0 - option_width / 2.0;
                    let y = start_y + (i as f32 * MENU_OPTION_SPACING);
                    
                    // Destacar opção selecionada
                    let color = if i == self.menu_selection {
                        BLACK
                    } else {
                        DARKGRAY
                    };
                    
                    // Efeito especial para easter egg no título
                    let title_color = if self.is_easter_egg() {
                        // Cor dourada para easter egg
                        Color::new(0.85, 0.65, 0.13, 1.0)
                    } else {
                        BLACK
                    };
                    
                    // Atualizar cor do título se for easter egg
                    if i == 0 && self.is_easter_egg() {
                        draw_text(
                            title,
                            screen_width() / 2.0 - title_width / 2.0,
                            screen_height() / 2.0 - 180.0,
                            MENU_TITLE_SIZE,
                            title_color,
                        );
                    }
                    
                    // Indicador de seleção com animação (piscar)
                    if i == self.menu_selection {
                        // Animação: piscar baseado em seno (garantir que o tempo está sendo atualizado)
                        // Usar módulo para evitar overflow e manter animação suave
                        let anim_time = self.menu_animation_time % (2.0 * std::f32::consts::PI);
                        let alpha = (anim_time.sin() * 0.4 + 0.6).clamp(0.3, 1.0);
                        let indicator_color = Color::new(0.0, 0.0, 0.0, alpha);
                        
                        // Desenhar indicador animado
                        draw_text(">", x - MENU_INDICATOR_OFFSET, y, MENU_OPTION_SIZE, indicator_color);
                        
                        // Optional: draw highlight line below selected option
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
                    
                    // Badge "NEW" para Continue se há novo save
                    if i == 0 && self.has_new_save {
                        let new_text = "NEW";
                        draw_text(new_text, x + option_width + 10.0, y, 20.0, RED);
                    }
                    
                    // Badge "NEW" para Versus se não foi jogado
                    if i == 2 && !self.versus_played {
                        let new_text = "NEW";
                        draw_text(new_text, x + option_width + 10.0, y, 20.0, GREEN);
                    }
                }
                
                // Calcular posição do último item do menu para evitar conflito
                let last_option_y = start_y + ((menu_options.len() - 1) as f32 * MENU_OPTION_SPACING);
                let last_option_height = MENU_OPTION_SIZE;
                let instructions_y = last_option_y + last_option_height + 40.0; // Espaçamento de 40px após o último item
                
                // Instructions (smaller and more discrete) - posicionadas após o último item do menu
                let instructions = "ARROWS/WASD: Navigate | ENTER/SPACE: Select";
                let inst_width = measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    instructions_y,
                    MENU_INSTRUCTION_SIZE,
                    GRAY,
                );
                
                // Versão do jogo (canto inferior esquerdo)
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
                // Tela de entrada de nome
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
                
                // Campo de entrada
                let input_label = "Name:";
                let input_size = 32.0;
                draw_text(
                    input_label,
                    screen_width() / 2.0 - 200.0,
                    screen_height() / 2.0 - 50.0,
                    input_size,
                    BLACK,
                );
                
                // Mostrar nome digitado
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
                
                // Mensagem de validação
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
                
                // Efeito especial se o nome for guicybercode
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
                
                // Instruções
                let instructions = "ENTER: Confirm | ESC: Cancel";
                let inst_width = measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
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
                
                // Conteúdo das páginas
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
                        BLACK // Título da seção
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
                
                // Número da página
                let page_text = format!("{}/{}", self.tutorial_page + 1, TUTORIAL_PAGE_COUNT);
                let page_width = measure_text(&page_text, None, 24, 1.0).width;
                draw_text(
                    &page_text,
                    screen_width() / 2.0 - page_width / 2.0,
                    screen_height() - 100.0,
                    24.0,
                    GRAY,
                );
                
                // Instruções
                let instructions = "LEFT/RIGHT: Navigate | ENTER: Next/Start | ESC: Skip";
                let inst_width = measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 40.0,
                    MENU_INSTRUCTION_SIZE,
                    GRAY,
                );
            }
            GameState::ContinueMenu => {
                // Menu de continue
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
                    
                    // Destacar slot selecionado
                    let color = if slot_idx == self.continue_selection {
                        BLACK
                    } else {
                        DARKGRAY
                    };
                    
                    // Indicador de seleção
                    if slot_idx == self.continue_selection {
                        draw_text(">", 100.0, y, 32.0, BLACK);
                    }
                    
                    // Informações do slot
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
                            if save_data.player_name.is_empty() { "Unknown" } else { &save_data.player_name }
                        );
                        draw_text(&slot_info, 150.0, y, 24.0, color);
                    } else {
                        let empty_text = format!("Slot {}: Empty Slot", slot_num);
                        draw_text(&empty_text, 150.0, y, 24.0, color);
                    }
                }
                
                // Modo de confirmação de delete
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
                    // Instruções
                    let instructions = "ENTER: Load | DELETE: Erase | ESC: Back";
                    let inst_width = measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
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
                // Exit confirmation screen
                let confirm_text = "ARE YOU SURE YOU WANT TO EXIT?";
                let confirm_size = 42.0;
                let confirm_width = measure_text(confirm_text, None, confirm_size as u16, 1.0).width;
                draw_text(
                    confirm_text,
                    screen_width() / 2.0 - confirm_width / 2.0,
                    screen_height() / 2.0 - 100.0,
                    confirm_size,
                    BLACK,
                );
                
                // Confirmation options
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
                // Title
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
                
                // Opções de configuração
                let option_size = 30.0;
                let start_y = 180.0;
                let spacing = 60.0;
                
                // Available resolutions (display strings)
                let resolution_names = ["800x600", "1024x768", "1280x720"];
                
                // Ensure resolution_index is within bounds
                let safe_resolution_index = self.resolution_index.min(RESOLUTIONS.len().saturating_sub(1));
                
                // Sound
                let sound_text = format!("SOUND: {}", if self.sound_enabled { "ON" } else { "OFF" });
                let sound_color = if self.settings_selection == 0 { BLACK } else { GRAY };
                let sound_width = measure_text(&sound_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 0 {
                    draw_text(">", screen_width() / 2.0 - sound_width / 2.0 - 30.0, start_y, option_size, BLACK);
                    draw_text("<", screen_width() / 2.0 + sound_width / 2.0 + 10.0, start_y, option_size, BLACK);
                }
                draw_text(&sound_text, screen_width() / 2.0 - sound_width / 2.0, start_y, option_size, sound_color);
                
                // Resolution
                let res_text = format!("RESOLUTION: {}", resolution_names[safe_resolution_index]);
                let res_color = if self.settings_selection == 1 { BLACK } else { GRAY };
                let res_width = measure_text(&res_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 1 {
                    draw_text("<", screen_width() / 2.0 - res_width / 2.0 - 30.0, start_y + spacing, option_size, BLACK);
                    draw_text(">", screen_width() / 2.0 + res_width / 2.0 + 10.0, start_y + spacing, option_size, BLACK);
                }
                draw_text(&res_text, screen_width() / 2.0 - res_width / 2.0, start_y + spacing, option_size, res_color);
                
                // Controls (display only)
                let controls_text = "CONTROLS";
                let controls_color = if self.settings_selection == 2 { BLACK } else { GRAY };
                let controls_width = measure_text(controls_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 2 {
                    draw_text(">", screen_width() / 2.0 - controls_width / 2.0 - 30.0, start_y + spacing * 2.0, option_size, BLACK);
                }
                draw_text(controls_text, screen_width() / 2.0 - controls_width / 2.0, start_y + spacing * 2.0, option_size, controls_color);
                
                // Show controls if selected
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
                
                // Back
                let back_text = "BACK";
                let back_color = if self.settings_selection == 3 { BLACK } else { GRAY };
                let back_width = measure_text(back_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 3 {
                    draw_text(">", screen_width() / 2.0 - back_width / 2.0 - 30.0, start_y + spacing * 4.5, option_size, BLACK);
                }
                draw_text(back_text, screen_width() / 2.0 - back_width / 2.0, start_y + spacing * 4.5, option_size, back_color);
                
                // Instructions
                let instructions = "Use ARROWS to navigate and adjust, ENTER to confirm, ESC to go back";
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
            GameState::Credits => {
                // Título
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
                
                // Informações de créditos organizadas
                let start_y = 140.0;
                let spacing = 28.0;
                let section_spacing = 40.0;
                let mut current_y = start_y;
                
                // Título do jogo
                let game_title = "JUMP QUEST";
                let game_title_size = 36.0;
                let game_title_width = measure_text(game_title, None, game_title_size as u16, 1.0).width;
                draw_text(
                    game_title,
                    screen_width() / 2.0 - game_title_width / 2.0,
                    current_y,
                    game_title_size,
                    BLACK,
                );
                current_y += section_spacing;
                
                // Made by
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
                
                // Developer name - using ASCII stylized version
                let developer = "guicybercode";
                let developer_size = 32.0;
                let developer_width = measure_text(developer, None, developer_size as u16, 1.0).width;
                draw_text(
                    developer,
                    screen_width() / 2.0 - developer_width / 2.0,
                    current_y,
                    developer_size,
                    BLACK,
                );
                current_y += section_spacing;
                
                // Seção: Technology Stack
                let stack_title = "Technology Stack";
                let stack_title_size = 24.0;
                let stack_title_width = measure_text(stack_title, None, stack_title_size as u16, 1.0).width;
                draw_text(
                    stack_title,
                    screen_width() / 2.0 - stack_title_width / 2.0,
                    current_y,
                    stack_title_size,
                    BLACK,
                );
                current_y += spacing;
                
                // Detalhes da stack
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
                
                // Seção: Game Info
                let info_title = "Game Information";
                let info_title_size = 24.0;
                let info_title_width = measure_text(info_title, None, info_title_size as u16, 1.0).width;
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
                
                // Instruções para voltar (no final)
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
                // Otimização: só desenhar objetos visíveis na tela
                let screen_left = self.camera.x - COLLISION_MARGIN; // Margem extra
                let screen_right = self.camera.x + screen_width() + COLLISION_MARGIN;
                let screen_top = self.camera.y - COLLISION_MARGIN;
                let screen_bottom = self.camera.y + screen_height() + COLLISION_MARGIN;
                
                // Desenhar plataformas (só as visíveis)
                for platform in &self.platforms {
                    if platform.x + platform.width >= screen_left 
                        && platform.x <= screen_right
                        && platform.y + platform.height >= screen_top
                        && platform.y <= screen_bottom
                    {
                        platform.draw(self.camera.x, self.camera.y);
                    }
                }
                
                // Desenhar checkpoints (só os visíveis)
                for checkpoint in &self.checkpoints {
                    if checkpoint.x >= screen_left 
                        && checkpoint.x <= screen_right
                        && checkpoint.y >= screen_top
                        && checkpoint.y <= screen_bottom
                    {
                        checkpoint.draw(self.camera.x, self.camera.y);
                    }
                }
                
                // Desenhar moedas (só as não coletadas e visíveis)
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
                
                // Desenhar inimigos (só os vivos e visíveis)
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
                
                // Desenhar jogador (sempre visível)
                self.player.draw(self.camera.x, self.camera.y);
                
                // UI - Contador de moedas, fase e tempo
                let time_seconds = self.time_remaining as u32;
                let time_text = format!("{}", time_seconds);
                
                // Cor do tempo: vermelho se menos de 30 segundos, amarelo se menos de 60, preto caso contrário
                let time_color = if self.time_remaining < TIME_WARNING_RED {
                    RED
                } else if self.time_remaining < TIME_WARNING_YELLOW {
                    YELLOW
                } else {
                    BLACK
                };
                
                // Mostrar nome do jogador
                let player_name_display = if self.player_name.is_empty() {
                    "Player"
                } else {
                    &self.player_name
                };
                draw_text(
                    player_name_display,
                    10.0,
                    30.0,
                    24.0,
                    BLACK,
                );
                
                draw_text(
                    &format!("Level: {} | Coins: {}/{} | Time: {}s", 
                        self.current_level, self.coins_collected, self.total_coins, time_seconds),
                    10.0,
                    60.0,
                    30.0,
                    BLACK,
                );
                
                // Show score
                let score_text = format!("Score: {}", self.score);
                draw_text(
                    &score_text,
                    10.0,
                    100.0,
                    28.0,
                    BLACK,
                );
                
                // Desenhar vidas
                let lives_text = format!("Lives: {}", self.lives);
                draw_text(
                    &lives_text,
                    10.0,
                    130.0,
                    28.0,
                    if self.lives <= 1 { RED } else { BLACK },
                );
                
                // Mostrar tempo em destaque no canto superior direito
                let time_width = measure_text(&time_text, None, 40u16, 1.0).width;
                draw_text(
                    &time_text,
                    screen_width() - time_width - 20.0,
                    40.0,
                    40.0,
                    time_color,
                );
                
                // Fadein ao iniciar fase (overlay preto que vai sumindo)
                if self.level_start_fade_timer > 0.0 {
                    let fade_progress = self.level_start_fade_timer / 1.5; // 1.0 a 0.0
                    let fade_alpha = fade_progress.min(1.0);
                    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, fade_alpha));
                }
            }
            GameState::Versus => {
                // Desenhar plataformas
                let camera_x = self.camera.x;
                let camera_y = self.camera.y;
                for platform in &self.versus_platforms {
                    platform.draw(camera_x, camera_y);
                }
                
                // Desenhar players
                if self.respawn_timer_p1 <= 0.0 {
                    self.player.draw_vs(camera_x, camera_y, true);
                }
                
                if let Some(ref p2) = self.player2 {
                    if self.respawn_timer_p2 <= 0.0 {
                        p2.draw_vs(camera_x, camera_y, false);
                    }
                }
                
                // Desenhar pontuações no topo (kills e pontos)
                let p1_score_text = format!("P1: {} kills | {} pts", self.player1_score, self.player1_points);
                let p2_score_text = format!("P2: {} kills | {} pts", self.player2_score, self.player2_points);
                
                draw_text(&p1_score_text, 20.0, 30.0, 24.0, BLACK);
                
                let p2_width = measure_text(&p2_score_text, None, 24u16, 1.0).width;
                draw_text(&p2_score_text, screen_width() - p2_width - 20.0, 30.0, 24.0, DARKGRAY);
                
                // Desenhar timer no topo central (apenas segundos)
                let time_text = format!("{}", self.versus_time_remaining as u32);
                let time_width = measure_text(&time_text, None, 28u16, 1.0).width;
                
                // Cor do timer (vermelho quando < 60 segundos)
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
                
                // Instruções
                let instructions = "P1: WASD | P2: Arrow Keys | ESC: Menu";
                let inst_width = measure_text(instructions, None, 16u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 30.0,
                    16.0,
                    GRAY,
                );
                
                // Fadein ao iniciar fase (overlay preto que vai sumindo)
                if self.level_start_fade_timer > 0.0 {
                    let fade_progress = self.level_start_fade_timer / 1.5; // 1.0 a 0.0
                    let fade_alpha = fade_progress.min(1.0);
                    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, fade_alpha));
                }
            }
            GameState::VersusEnd => {
                // Tela de resultados finais
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
                
                // Determinar vencedor baseado em pontos (não kills)
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
                
                // Scores finais (kills e pontos)
                let score_size = 28.0;
                let p1_final_text = format!("Player 1: {} kills | {} points", self.player1_score, self.player1_points);
                let p2_final_text = format!("Player 2: {} kills | {} points", self.player2_score, self.player2_points);
                
                let p1_final_width = measure_text(&p1_final_text, None, score_size as u16, 1.0).width;
                let p2_final_width = measure_text(&p2_final_text, None, score_size as u16, 1.0).width;
                
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
                
                // Instruções para voltar
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
                // Title
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
                
                // Desenhar as 4 fases
                let spacing = 100.0;
                let start_x = screen_width() / 2.0 - (spacing * 3.0) / 2.0;
                let center_y = screen_height() / 2.0;
                let level_names = vec!["Level 1", "Level 2", "Level 3", "Level 4"];
                
                for i in 0..4 {
                    let x = start_x + (i as f32 * spacing);
                    let is_selected = i == self.level_selection;
                    let is_unlocked = self.unlocked_levels[i];
                    
                    // Desenhar círculo da fase
                    let circle_color = if !is_unlocked {
                        // Fase bloqueada - cinza escuro
                        DARKGRAY
                    } else if is_selected {
                        BLACK
                    } else {
                        GRAY
                    };
                    let circle_radius = if is_selected { 35.0 } else { 30.0 };
                    draw_circle(x, center_y, circle_radius, circle_color);
                    draw_circle_lines(x, center_y, circle_radius, 3.0, if is_selected { WHITE } else { BLACK });
                    
                    // Número da fase
                    let num_text = format!("{}", i + 1);
                    let num_size = 36.0;
                    let num_width = measure_text(&num_text, None, num_size as u16, 1.0).width;
                    draw_text(
                        &num_text,
                        x - num_width / 2.0,
                        center_y + num_size / 3.0,
                        num_size,
                        if is_unlocked {
                            if is_selected { WHITE } else { BLACK }
                        } else {
                            GRAY
                        },
                    );
                    
                    // Level name and info below
                    let (difficulty, coin_count, difficulty_color) = self.get_level_info(i + 1);
                    
                    if is_unlocked {
                        let name_size = 18.0;
                        let name_text = level_names[i];
                        let name_width = measure_text(name_text, None, name_size as u16, 1.0).width;
                        draw_text(
                            name_text,
                            x - name_width / 2.0,
                            center_y + 60.0,
                            name_size,
                            if is_selected { BLACK } else { GRAY },
                        );
                        
                        // Mostrar dificuldade com cor
                        let diff_text = difficulty;
                        let diff_width = measure_text(&diff_text, None, 16, 1.0).width;
                        draw_text(
                            &diff_text,
                            x - diff_width / 2.0,
                            center_y + 82.0,
                            16.0,
                            difficulty_color,
                        );
                        
                        // Mostrar número de moedas
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
                        // Locked indicator
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
                
                // Instructions
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
                // Desenhar o jogo por trás (estado congelado)
                // Primeiro, desenhar tudo do jogo normalmente
                let screen_left = self.camera.x - COLLISION_MARGIN;
                let screen_right = self.camera.x + screen_width() + COLLISION_MARGIN;
                let screen_top = self.camera.y - COLLISION_MARGIN;
                let screen_bottom = self.camera.y + screen_height() + COLLISION_MARGIN;
                
                // Desenhar plataformas
                for platform in &self.platforms {
                    if platform.x + platform.width >= screen_left 
                        && platform.x <= screen_right
                        && platform.y + platform.height >= screen_top
                        && platform.y <= screen_bottom
                    {
                        platform.draw(self.camera.x, self.camera.y);
                    }
                }
                
                // Desenhar checkpoints
                for checkpoint in &self.checkpoints {
                    if checkpoint.x >= screen_left 
                        && checkpoint.x <= screen_right
                        && checkpoint.y >= screen_top
                        && checkpoint.y <= screen_bottom
                    {
                        checkpoint.draw(self.camera.x, self.camera.y);
                    }
                }
                
                // Desenhar moedas
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
                
                // Desenhar inimigos
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
                
                // Desenhar player
                self.player.draw(self.camera.x, self.camera.y);
                
                // Desenhar HUD
                let time_seconds = self.time_remaining as u32;
                let time_text = format!("Time: {}s", time_seconds);
                let time_color = if self.time_remaining < TIME_WARNING_RED {
                    RED
                } else if self.time_remaining < TIME_WARNING_YELLOW {
                    ORANGE
                } else {
                    BLACK
                };
                
                draw_text(
                    &format!("Level: {} | Coins: {}/{} | Time: {}s", 
                        self.current_level, self.coins_collected, self.total_coins, time_seconds),
                    10.0,
                    60.0,
                    30.0,
                    BLACK,
                );
                
                let score_text = format!("Score: {}", self.score);
                draw_text(&score_text, 10.0, 100.0, 28.0, BLACK);
                
                let lives_text = format!("Lives: {}", self.lives);
                draw_text(
                    &lives_text,
                    10.0,
                    130.0,
                    28.0,
                    if self.lives <= 1 { RED } else { BLACK },
                );
                
                let player_name_display = if self.player_name.is_empty() {
                    "Player"
                } else {
                    &self.player_name
                };
                draw_text(player_name_display, 10.0, 30.0, 24.0, BLACK);
                
                let time_width = measure_text(&time_text, None, 40u16, 1.0).width;
                draw_text(
                    &time_text,
                    screen_width() - time_width - 20.0,
                    40.0,
                    40.0,
                    time_color,
                );
                
                // Overlay escuro semi-transparente para efeito de blur/desfoque
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), 
                    Color::new(0.0, 0.0, 0.0, 0.6));
                
                // Menu de pausa
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
                    let option_width = measure_text(option, None, MENU_OPTION_SIZE as u16, 1.0).width;
                    let x = screen_width() / 2.0 - option_width / 2.0;
                    let y = start_y + (i as f32 * MENU_OPTION_SPACING);
                    
                    let color = if i == self.pause_selection {
                        WHITE
                    } else {
                        LIGHTGRAY
                    };
                    
                    // Indicador de seleção
                    if i == self.pause_selection {
                        draw_text(">", x - MENU_INDICATOR_OFFSET, y, MENU_OPTION_SIZE, WHITE);
                    }
                    
                    draw_text(option, x, y, MENU_OPTION_SIZE, color);
                }
                
                // Instruções
                let instructions = "ARROWS/WASD: Navigate | ENTER: Select | P/ESC: Resume";
                let inst_width = measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 40.0,
                    MENU_INSTRUCTION_SIZE,
                    LIGHTGRAY,
                );
            }
            GameState::Respawn => {
                // Tela de respawn com fadeout
                let time_remaining = self.respawn_timer;
                let total_time = 3.0;
                let progress = 1.0 - (time_remaining / total_time); // 0.0 a 1.0
                
                // Fadeout do fundo (preto com alpha aumentando)
                let bg_alpha = (progress * 0.8).min(0.8);
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, bg_alpha));
                
                // Desenhar personagem com fadeout (no centro)
                let player_size = 64.0;
                let player_x = screen_width() / 2.0 - player_size / 2.0;
                let player_y = screen_height() / 2.0 - player_size / 2.0 - 50.0;
                
                // Alpha do personagem diminui (fadeout)
                let player_alpha = (1.0 - progress * 1.5).max(0.0);
                let player_color = Color::new(0.0, 0.0, 0.0, player_alpha);
                
                // Desenhar personagem simplificado (retângulo preto)
                draw_rectangle(player_x, player_y, player_size, player_size, player_color);
                draw_rectangle_lines(player_x, player_y, player_size, player_size, 2.0, Color::new(1.0, 1.0, 1.0, player_alpha));
                
                // Olhos do personagem
                let eye_size = 6.0;
                let eye_y = player_y + 20.0;
                draw_circle(player_x + 20.0, eye_y, eye_size, Color::new(1.0, 1.0, 1.0, player_alpha));
                draw_circle(player_x + 44.0, eye_y, eye_size, Color::new(1.0, 1.0, 1.0, player_alpha));
                
                // Texto de vidas restantes
                let lives_text = format!("Lives: {}", self.lives);
                let lives_size = 48.0;
                let lives_width = measure_text(&lives_text, None, lives_size as u16, 1.0).width;
                let lives_alpha = 1.0 - (progress * 0.5).min(0.5); // Fade mais lento que o personagem
                let lives_color = Color::new(1.0, 1.0, 1.0, lives_alpha);
                
                draw_text(
                    &lives_text,
                    screen_width() / 2.0 - lives_width / 2.0,
                    screen_height() / 2.0 + 80.0,
                    lives_size,
                    lives_color,
                );
                
                // Contagem regressiva
                let countdown = (time_remaining.ceil() as u32).max(1);
                let countdown_text = format!("Respawn in {}...", countdown);
                let countdown_size = 32.0;
                let countdown_width = measure_text(&countdown_text, None, countdown_size as u16, 1.0).width;
                
                draw_text(
                    &countdown_text,
                    screen_width() / 2.0 - countdown_width / 2.0,
                    screen_height() / 2.0 + 140.0,
                    countdown_size,
                    lives_color,
                );
            }
            GameState::GameOver => {
                // Fadeout no game over (overlay preto que vai aparecendo)
                let fade_progress = if self.game_over_fade_timer > 0.0 {
                    (2.0 - self.game_over_fade_timer) / 2.0 // 0.0 a 1.0
                } else {
                    1.0 // Fade completo
                };
                let fade_alpha = fade_progress.min(1.0);
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, fade_alpha));
                
                // Game over screen (só mostrar após fadeout completo)
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
                    
                    // Show score
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
                    
                    // Show coins collected
                    let coins_text = format!("Coins: {}/{}", self.coins_collected, self.total_coins);
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
                    let restart_width = measure_text(restart_text, None, restart_size as u16, 1.0).width;
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
                // Level complete screen
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
                
                // Show score
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
                
                // Show coins collected
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
                
                // Mostrar vidas no Level Complete
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
                
                // Unlock message
                if self.current_level < 4 
                    && self.current_level < self.unlocked_levels.len() 
                    && self.unlocked_levels[self.current_level] 
                {
                    let unlock_text = format!("Level {} unlocked!", self.current_level + 1);
                    let unlock_size = 24.0;
                    let unlock_width = measure_text(&unlock_text, None, unlock_size as u16, 1.0).width;
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
                let continue_width = measure_text(continue_text, None, continue_size as u16, 1.0).width;
                draw_text(
                    continue_text,
                    screen_width() / 2.0 - continue_width / 2.0,
                    screen_height() / 2.0 + 60.0,
                    continue_size,
                    GRAY,
                );
            }
        }
        
        // Desenhar overlay de transição no final (sobre todas as telas)
        self.draw_transition();
    }
}

