use macroquad::prelude::*;
use crate::constants::*;

pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub vel_x: f32,
    pub vel_y: f32,  // Velocidade vertical para física correta
    pub alive: bool,
    pub on_ground: bool,
}

impl Enemy {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            width: ENEMY_WIDTH,
            height: ENEMY_HEIGHT,
            vel_x: -ENEMY_SPEED, // Velocidade inicial horizontal
            vel_y: 0.0,          // Velocidade inicial vertical
            alive: true,
            on_ground: true, // Começar no chão para evitar queda imediata
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.alive {
            return;
        }
        
        // Mover horizontalmente primeiro
        self.x += self.vel_x * dt;
        
        // Aplicar gravidade na velocidade (não na posição diretamente)
        if !self.on_ground {
            self.vel_y += ENEMY_GRAVITY * dt; // Gravidade acelera a queda
        }
        
        // Limitar velocidade terminal
        if self.vel_y > TERMINAL_VELOCITY {
            self.vel_y = TERMINAL_VELOCITY;
        }
        
        // Atualizar posição vertical baseado na velocidade
        self.y += self.vel_y * dt;
        
        // Resetar flag de chão (será atualizado na colisão)
        self.on_ground = false;
    }

    #[inline]
    pub fn check_platform_collision(&mut self, platform: &crate::platform::Platform) {
        if !self.alive {
            return;
        }
        
        // Verificação AABB direta
        if self.x < platform.x + platform.width
            && self.x + self.width > platform.x
            && self.y < platform.y + platform.height
            && self.y + self.height > platform.y
        {
            // Calcular sobreposições
            let overlap_top = (self.y + self.height) - platform.y;
            let overlap_left = (self.x + self.width) - platform.x;
            let overlap_right = (platform.x + platform.width) - self.x;
            
            // Se overlap vertical é menor, é colisão por cima (pousando na plataforma)
            if overlap_top < overlap_left.min(overlap_right) && overlap_top < PLATFORM_COLLISION_THRESHOLD && self.vel_y >= 0.0 {
                self.y = platform.y - self.height;
                self.vel_y = 0.0; // Resetar velocidade vertical ao pousar
                self.on_ground = true;
            } else {
                // Colisão lateral - inverter direção
                self.vel_x = -self.vel_x;
                if self.vel_x > 0.0 {
                    self.x = platform.x + platform.width + 1.0;
                } else {
                    self.x = platform.x - self.width - 1.0;
                }
            }
        }
    }
    
    // Verificar se há plataforma à frente e abaixo (para não cair)
    #[inline]
    pub fn check_edge(&mut self, platforms: &[crate::platform::Platform]) {
        if !self.alive || !self.on_ground {
            return;
        }
        
        // Verificar se há plataforma abaixo e à frente (na direção do movimento)
        let check_offset_x = if self.vel_x > 0.0 {
            self.width + 2.0
        } else {
            -2.0
        };
        
        let check_x = self.x + check_offset_x;
        let check_y = self.y + self.height + 15.0;
        
        let mut found_platform_below = false;
        
        for platform in platforms {
            // Verificação direta (mais rápida)
            if check_x >= platform.x 
                && check_x <= platform.x + platform.width
                && check_y >= platform.y 
                && check_y <= platform.y + platform.height
            {
                found_platform_below = true;
                break;
            }
        }
        
        if !found_platform_below {
            self.vel_x = -self.vel_x;
            if self.vel_x > 0.0 {
                self.x += 2.0;
            } else {
                self.x -= 2.0;
            }
        }
    }

    pub fn check_ground_collision(&mut self, ground_y: f32) {
        if !self.alive {
            return;
        }
        
        if self.y + self.height >= ground_y {
            self.y = ground_y - self.height;
            self.vel_y = 0.0; // Resetar velocidade vertical ao tocar o chão
            self.on_ground = true;
        }
    }

    /// Resultado da colisão jogador-inimigo
    /// - None: sem colisão
    /// - Some(true): jogador morreu
    /// - Some(false): inimigo morreu (jogador deve quicar)
    pub fn check_player_collision(
        &mut self,
        player_x: f32,
        player_y: f32,
        player_w: f32,
        player_h: f32,
        player_vel_y: f32,
    ) -> Option<bool> {
        if !self.alive {
            return None;
        }
        
        if self.check_collision(player_x, player_y, player_w, player_h) {
            // Calcular a posição relativa do jogador em relação ao inimigo
            let player_bottom = player_y + player_h;
            let enemy_top = self.y;
            let enemy_center_y = self.y + self.height / 2.0;
            
            // Margem de tolerância para considerar "em cima" do inimigo
            // O jogador está "em cima" se:
            // 1. Está caindo (vel_y > 0) ou quase parado (vel_y >= -50)
            // 2. A parte de baixo do jogador está na metade superior do inimigo
            let is_on_top = player_vel_y >= -50.0 && player_bottom <= enemy_center_y + 10.0;
            
            // Também considerar se o jogador está acima do topo do inimigo (caindo sobre ele)
            let is_falling_on_top = player_vel_y > 0.0 && player_bottom <= enemy_top + self.height * 0.7;
            
            if is_on_top || is_falling_on_top {
                self.alive = false;
                return Some(false); // Inimigo morreu, jogador deve quicar
            }
            
            // Caso contrário, mata o jogador
            return Some(true);
        }
        None
    }

    fn check_collision(&self, other_x: f32, other_y: f32, other_w: f32, other_h: f32) -> bool {
        other_x < self.x + self.width
            && other_x + other_w > self.x
            && other_y < self.y + self.height
            && other_y + other_h > self.y
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32) {
        if !self.alive {
            return;
        }
        
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        
        // Desenhar inimigo em preto e branco (Goomba simples)
        draw_rectangle(screen_x, screen_y, self.width, self.height, GRAY);
        draw_rectangle_lines(screen_x, screen_y, self.width, self.height, 2.0, BLACK);
        
        // Olhos simples
        draw_circle(screen_x + 6.0, screen_y + 8.0, 2.0, BLACK);
        draw_circle(screen_x + 18.0, screen_y + 8.0, 2.0, BLACK);
    }
}

pub fn create_level_enemies(level: usize) -> Vec<Enemy> {
    let mut enemies = Vec::new();
    
    match level {
        1 => {
            // Fase 1
            enemies.push(Enemy::new(300.0, 420.0));
            enemies.push(Enemy::new(500.0, 370.0));
            enemies.push(Enemy::new(700.0, 320.0));
            enemies.push(Enemy::new(900.0, 420.0));
            enemies.push(Enemy::new(1200.0, 370.0));
            enemies.push(Enemy::new(1400.0, 320.0));
            enemies.push(Enemy::new(1600.0, 370.0));
            enemies.push(Enemy::new(2100.0, 420.0));
            enemies.push(Enemy::new(2300.0, 370.0));
        }
        2 => {
            // Fase 2 - Mais inimigos
            enemies.push(Enemy::new(200.0, 470.0));
            enemies.push(Enemy::new(350.0, 420.0));
            enemies.push(Enemy::new(500.0, 370.0));
            enemies.push(Enemy::new(650.0, 320.0));
            enemies.push(Enemy::new(800.0, 270.0));
            enemies.push(Enemy::new(950.0, 220.0));
            enemies.push(Enemy::new(1100.0, 270.0));
            enemies.push(Enemy::new(1250.0, 320.0));
            enemies.push(Enemy::new(1400.0, 370.0));
            enemies.push(Enemy::new(1550.0, 420.0));
            enemies.push(Enemy::new(1700.0, 470.0));
        }
        3 => {
            // Fase 3
            enemies.push(Enemy::new(200.0, 420.0));
            enemies.push(Enemy::new(500.0, 470.0));
            enemies.push(Enemy::new(700.0, 420.0));
            enemies.push(Enemy::new(900.0, 370.0));
            enemies.push(Enemy::new(1150.0, 320.0));
            enemies.push(Enemy::new(1350.0, 270.0));
            enemies.push(Enemy::new(1650.0, 320.0));
            enemies.push(Enemy::new(1900.0, 370.0));
            enemies.push(Enemy::new(2100.0, 420.0));
            enemies.push(Enemy::new(2350.0, 470.0));
            enemies.push(Enemy::new(2550.0, 420.0));
            enemies.push(Enemy::new(400.0, 270.0));
            enemies.push(Enemy::new(600.0, 220.0));
        }
        4 => {
            // Fase 4 - Mais inimigos
            enemies.push(Enemy::new(150.0, 470.0));
            enemies.push(Enemy::new(300.0, 420.0));
            enemies.push(Enemy::new(450.0, 470.0));
            enemies.push(Enemy::new(600.0, 420.0));
            enemies.push(Enemy::new(750.0, 470.0));
            enemies.push(Enemy::new(900.0, 420.0));
            enemies.push(Enemy::new(1100.0, 320.0));
            enemies.push(Enemy::new(1300.0, 270.0));
            enemies.push(Enemy::new(1500.0, 220.0));
            enemies.push(Enemy::new(1700.0, 170.0));
            enemies.push(Enemy::new(1900.0, 370.0));
            enemies.push(Enemy::new(2100.0, 320.0));
            enemies.push(Enemy::new(2300.0, 270.0));
            enemies.push(Enemy::new(2500.0, 220.0));
            enemies.push(Enemy::new(2700.0, 320.0));
        }
        _ => {}
    }
    
    enemies
}
