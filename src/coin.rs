use macroquad::prelude::*;
use crate::constants::*;

pub struct Coin {
    pub x: f32,
    pub y: f32,
    pub collected: bool,
    pub rotation: f32,
}

impl Coin {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            collected: false,
            rotation: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.collected {
            // Animação de rotação
            self.rotation += COIN_ROTATION_SPEED * dt;
        }
    }

    pub fn check_collection(
        &mut self,
        player_x: f32,
        player_y: f32,
        player_w: f32,
        player_h: f32,
    ) -> bool {
        if self.collected {
            return false;
        }
        
        if player_x < self.x + COIN_SIZE
            && player_x + player_w > self.x
            && player_y < self.y + COIN_SIZE
            && player_y + player_h > self.y
        {
            self.collected = true;
            return true;
        }
        false
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32) {
        if self.collected {
            return;
        }
        
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        
        // Desenhar moeda em preto e branco (círculo)
        draw_circle(screen_x + 8.0, screen_y + 8.0, 8.0, WHITE);
        draw_circle_lines(screen_x + 8.0, screen_y + 8.0, 8.0, 2.0, BLACK);
        
        // Linha central para efeito de rotação
        let center_x = screen_x + 8.0;
        let center_y = screen_y + 8.0;
        let end_x = center_x + 6.0 * self.rotation.cos();
        let end_y = center_y + 6.0 * self.rotation.sin();
        draw_line(center_x, center_y, end_x, end_y, 2.0, BLACK);
    }
}

pub fn create_level_coins(level: usize) -> Vec<Coin> {
    let mut coins = Vec::new();
    
    match level {
        1 => {
            // Fase 1
            coins.push(Coin::new(250.0, 420.0));
            coins.push(Coin::new(450.0, 370.0));
            coins.push(Coin::new(650.0, 320.0));
            coins.push(Coin::new(850.0, 420.0));
            coins.push(Coin::new(1150.0, 370.0));
            coins.push(Coin::new(1350.0, 320.0));
            coins.push(Coin::new(1550.0, 370.0));
            coins.push(Coin::new(1750.0, 270.0));
            coins.push(Coin::new(2050.0, 420.0));
            coins.push(Coin::new(2250.0, 370.0));
            coins.push(Coin::new(2450.0, 320.0));
        }
        2 => {
            // Fase 2
            coins.push(Coin::new(200.0, 470.0));
            coins.push(Coin::new(350.0, 420.0));
            coins.push(Coin::new(500.0, 370.0));
            coins.push(Coin::new(650.0, 320.0));
            coins.push(Coin::new(800.0, 270.0));
            coins.push(Coin::new(950.0, 220.0));
            coins.push(Coin::new(1100.0, 270.0));
            coins.push(Coin::new(1250.0, 320.0));
            coins.push(Coin::new(1400.0, 370.0));
            coins.push(Coin::new(1550.0, 420.0));
            coins.push(Coin::new(1700.0, 470.0));
            coins.push(Coin::new(1850.0, 370.0));
            coins.push(Coin::new(2200.0, 320.0));
            coins.push(Coin::new(2400.0, 270.0));
            coins.push(Coin::new(2600.0, 220.0));
        }
        3 => {
            // Fase 3
            coins.push(Coin::new(150.0, 420.0));
            coins.push(Coin::new(450.0, 470.0));
            coins.push(Coin::new(700.0, 420.0));
            coins.push(Coin::new(900.0, 370.0));
            coins.push(Coin::new(1150.0, 320.0));
            coins.push(Coin::new(1350.0, 270.0));
            coins.push(Coin::new(1650.0, 320.0));
            coins.push(Coin::new(1900.0, 370.0));
            coins.push(Coin::new(2100.0, 420.0));
            coins.push(Coin::new(2350.0, 470.0));
            coins.push(Coin::new(2550.0, 420.0));
            coins.push(Coin::new(350.0, 270.0));
            coins.push(Coin::new(550.0, 220.0));
            coins.push(Coin::new(750.0, 170.0));
            coins.push(Coin::new(1550.0, 170.0));
            coins.push(Coin::new(1750.0, 220.0));
        }
        4 => {
            // Fase 4
            coins.push(Coin::new(150.0, 470.0));
            coins.push(Coin::new(300.0, 420.0));
            coins.push(Coin::new(450.0, 470.0));
            coins.push(Coin::new(600.0, 420.0));
            coins.push(Coin::new(750.0, 470.0));
            coins.push(Coin::new(900.0, 420.0));
            coins.push(Coin::new(1050.0, 320.0));
            coins.push(Coin::new(1250.0, 270.0));
            coins.push(Coin::new(1450.0, 220.0));
            coins.push(Coin::new(1650.0, 170.0));
            coins.push(Coin::new(1850.0, 370.0));
            coins.push(Coin::new(2000.0, 320.0));
            coins.push(Coin::new(2150.0, 270.0));
            coins.push(Coin::new(2300.0, 220.0));
            coins.push(Coin::new(2450.0, 370.0));
            coins.push(Coin::new(2600.0, 320.0));
            coins.push(Coin::new(2750.0, 270.0));
        }
        _ => {}
    }
    
    coins
}

