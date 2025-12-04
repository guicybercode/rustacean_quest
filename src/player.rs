use macroquad::prelude::*;
use crate::constants::*;
use std::rc::Rc;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub on_ground: bool,
    pub facing_right: bool,
    pub sprite_texture_p1: Option<Rc<Texture2D>>, // Textura do sprite do Ferris P1 (laranja)
    pub sprite_texture_p2: Option<Rc<Texture2D>>, // Textura do sprite do Ferris P2 (escuro)
    pub animation_frame: usize, // Frame atual da animação (0-3)
    pub animation_timer: f32, // Timer para controlar a velocidade da animação
    pub walk_bounce_timer: f32, // Timer para efeito de bounce ao andar
}

impl Player {
    pub fn new(x: f32, y: f32, sprite_texture_p1: Option<Rc<Texture2D>>, sprite_texture_p2: Option<Rc<Texture2D>>) -> Self {
        Self {
            x,
            y,
            width: PLAYER_WIDTH,
            height: PLAYER_HEIGHT,
            vel_x: 0.0,
            vel_y: 0.0,
            on_ground: true, // Começar no chão
            facing_right: true,
            sprite_texture_p1,
            sprite_texture_p2,
            animation_frame: 0,
            animation_timer: 0.0,
            walk_bounce_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Aplicar gravidade
        if !self.on_ground {
            self.vel_y += GRAVITY * dt;
        }
        
        // Velocidade terminal
        if self.vel_y > TERMINAL_VELOCITY {
            self.vel_y = TERMINAL_VELOCITY;
        }
        
        // Atrito no chão
        if self.on_ground {
            self.vel_x *= PLAYER_FRICTION;
        }
        
        // Atualizar posição
        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;
        
        // Atualizar timer de bounce (será usado no draw para efeito visual)
        // O timer é atualizado sempre, mas só é usado visualmente quando está andando
        const WALK_BOUNCE_SPEED: f32 = 12.0; // Velocidade do bounce
        self.walk_bounce_timer += dt * WALK_BOUNCE_SPEED;
        
        // Resetar timer quando parar completamente
        if self.vel_x.abs() < 1.0 {
            self.walk_bounce_timer = 0.0;
        }
        
        // Resetar flag de chão (será atualizado na colisão)
        self.on_ground = false;
    }
    
    /// Atualiza a animação do player (deve ser chamado DEPOIS das colisões)
    pub fn update_animation(&mut self, dt: f32) {
        const ANIMATION_SPEED: f32 = 0.08; // Tempo entre frames (em segundos) - mais rápido para animação mais aparente
        const MIN_VELOCITY_FOR_ANIMATION: f32 = 5.0; // Velocidade mínima para animar
        
        // Verificar se está andando (no chão e com velocidade horizontal significativa)
        if self.on_ground && self.vel_x.abs() > MIN_VELOCITY_FOR_ANIMATION {
            // Está andando - animar (frames 0, 1, 2, 3 que correspondem a 1, 2, 3, 4 na imagem)
            self.animation_timer += dt;
            if self.animation_timer >= ANIMATION_SPEED {
                self.animation_timer = 0.0;
                self.animation_frame = (self.animation_frame + 1) % 4; // Ciclar entre 0-3 (frames 1-4)
            }
        } else {
            // Parado ou no ar - usar frame 0 (idle, primeiro frame)
            self.animation_frame = 0;
            self.animation_timer = 0.0;
        }
    }

    pub fn handle_movement(&mut self) {
        // Movimento horizontal
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.vel_x = -PLAYER_SPEED;
            self.facing_right = false;
        } else if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.vel_x = PLAYER_SPEED;
            self.facing_right = true;
        }
    }

    pub fn handle_movement_custom(&mut self, left: bool, right: bool) {
        // Movimento horizontal com controles customizados
        if left {
            self.vel_x = -PLAYER_SPEED;
            self.facing_right = false;
        } else if right {
            self.vel_x = PLAYER_SPEED;
            self.facing_right = true;
        }
    }

    pub fn handle_jump(&mut self) -> bool {
        // Pular apenas se estiver no chão (verificado após colisões)
        // Usar is_key_down para permitir pulos mais rápidos ao manter a tecla pressionada
        if is_key_down(KeyCode::Space) || is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            if self.on_ground {
                self.vel_y = JUMP_FORCE;
                self.on_ground = false;
                return true;
            }
        }
        false
    }

    pub fn handle_jump_custom(&mut self, jump_pressed: bool) -> bool {
        // Pular apenas se estiver no chão (verificado após colisões) com controle customizado
        // jump_pressed já vem de is_key_down, então permite pulos rápidos
        if jump_pressed {
            if self.on_ground {
                self.vel_y = JUMP_FORCE;
                self.on_ground = false;
                return true;
            }
        }
        false
    }

    pub fn check_platform_collision(&mut self, platform: &crate::platform::Platform) {
        if let Some((new_x, new_y, on_top)) = platform.get_collision_response(
            self.x,
            self.y,
            self.width,
            self.height,
            self.vel_y,
        ) {
            if on_top {
                // Colisão por cima da plataforma (piso)
                self.y = new_y;
                self.vel_y = 0.0;
                self.on_ground = true;
            } else {
                // Colisão lateral ou por baixo
                if (new_x - self.x).abs() > (new_y - self.y).abs() {
                    // Colisão lateral
                    self.x = new_x;
                    self.vel_x = 0.0;
                } else {
                    // Colisão por baixo (teto)
                    self.y = new_y;
                    self.vel_y = 0.0;
                }
            }
        }
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let mut screen_y = self.y - camera_y;
        
        // Aplicar efeito de bounce ao andar (mais aparente)
        const WALK_BOUNCE_AMOUNT: f32 = 2.0; // Quantidade de movimento vertical (em pixels) - mais visível
        if self.on_ground && self.vel_x.abs() > 5.0 {
            let bounce_offset = self.walk_bounce_timer.sin() * WALK_BOUNCE_AMOUNT;
            screen_y += bounce_offset; // Aplicar offset visual apenas
        }
        
        if let Some(texture) = &self.sprite_texture_p1 {
            // Usar sprite P1 (laranja) para single player
            // A imagem tem 4 frames lado a lado
            let sprite_width = texture.width() / 4.0; // Cada frame é 1/4 da largura
            let sprite_height = texture.height();
            
            // Calcular posição X do frame atual na textura
            let source_x = self.animation_frame as f32 * sprite_width;
            
            // Usar o frame atual da animação
            let source_rect = Rect::new(source_x, 0.0, sprite_width, sprite_height);
            
            // Parâmetros de desenho com flip horizontal (sem rotação, sem transparência)
            let params = DrawTextureParams {
                dest_size: Some(vec2(self.width, self.height)),
                source: Some(source_rect),
                rotation: 0.0, // Sem rotação
                flip_x: !self.facing_right,
                flip_y: false,
                pivot: None,
            };
            
            // Desenhar com cor branca (opacidade total, sem transparência)
            draw_texture_ex(&**texture, screen_x, screen_y, WHITE, params);
        } else {
            // Fallback: desenhar retângulo simples se a textura não carregou
            draw_rectangle(screen_x, screen_y, self.width, self.height, BLACK);
            draw_circle(screen_x + 10.0, screen_y + 10.0, 3.0, WHITE);
            draw_circle(screen_x + 22.0, screen_y + 10.0, 3.0, WHITE);
            draw_rectangle_lines(screen_x, screen_y, self.width, self.height, 2.0, WHITE);
        }
    }

    pub fn draw_vs(&self, camera_x: f32, camera_y: f32, is_player1: bool) {
        let screen_x = self.x - camera_x;
        let mut screen_y = self.y - camera_y;
        
        // Aplicar efeito de bounce ao andar (mais aparente)
        const WALK_BOUNCE_AMOUNT: f32 = 2.0; // Quantidade de movimento vertical (em pixels) - mais visível
        if self.on_ground && self.vel_x.abs() > 5.0 {
            let bounce_offset = self.walk_bounce_timer.sin() * WALK_BOUNCE_AMOUNT;
            screen_y += bounce_offset; // Aplicar offset visual apenas
        }
        
        // Escolher qual textura usar baseado no player
        let texture_opt = if is_player1 {
            &self.sprite_texture_p1
        } else {
            &self.sprite_texture_p2
        };
        
        if let Some(texture) = texture_opt {
            // Usar sprite apropriado (P1 ou P2)
            // A imagem tem 4 frames lado a lado
            let sprite_width = texture.width() / 4.0; // Cada frame é 1/4 da largura
            let sprite_height = texture.height();
            
            // Calcular posição X do frame atual na textura
            let source_x = self.animation_frame as f32 * sprite_width;
            
            // Usar o frame atual da animação
            let source_rect = Rect::new(source_x, 0.0, sprite_width, sprite_height);
            
            // Parâmetros de desenho com flip horizontal (sem rotação)
            let params = DrawTextureParams {
                dest_size: Some(vec2(self.width, self.height)),
                source: Some(source_rect),
                rotation: 0.0, // Sem rotação
                flip_x: !self.facing_right,
                flip_y: false,
                pivot: None,
            };
            
            // Desenhar com opacidade total (sem transparência)
            draw_texture_ex(&**texture, screen_x, screen_y, WHITE, params);
        } else {
            // Fallback: desenhar retângulo simples se a textura não carregou
            let color = if is_player1 { BLACK } else { DARKGRAY };
            draw_rectangle(screen_x, screen_y, self.width, self.height, color);
            draw_circle(screen_x + 10.0, screen_y + 10.0, 3.0, WHITE);
            draw_circle(screen_x + 22.0, screen_y + 10.0, 3.0, WHITE);
            draw_rectangle_lines(screen_x, screen_y, self.width, self.height, 2.0, WHITE);
        }
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    pub fn check_collision(&self, other: &Player) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub fn check_stomp(&self, other: &Player, self_vel_y: f32) -> bool {
        // Verifica se este player pulou em cima do outro
        if !self.check_collision(other) {
            return false;
        }
        
        let self_bottom = self.y + self.height;
        let other_top = other.y;
        let other_center_y = other.y + other.height / 2.0;
        
        // Player está "em cima" se:
        // 1. Está caindo ou quase parado (vel_y >= -50.0)
        // 2. O bottom do player está na metade superior do outro
        let is_on_top = self_vel_y >= -50.0 && self_bottom <= other_center_y + 10.0;
        
        // Também considerar se está caindo em cima
        let is_falling_on_top = self_vel_y > 0.0 && self_bottom <= other_top + other.height * 0.7;
        
        is_on_top || is_falling_on_top
    }
}

