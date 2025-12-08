use macroquad::prelude::*;
use crate::constants::*;

pub struct Checkpoint {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub activated: bool,
}

impl Checkpoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            width: CHECKPOINT_WIDTH,
            height: CHECKPOINT_HEIGHT,
            activated: false,
        }
    }

    pub fn check_activation(&mut self, player_x: f32, player_y: f32, player_w: f32, player_h: f32) -> bool {
        // Verificar se o jogador passou pelo checkpoint (colisão AABB)
        if !self.activated 
            && player_x < self.x + self.width
            && player_x + player_w > self.x
            && player_y < self.y + self.height
            && player_y + player_h > self.y
        {
            self.activated = true;
            return true;
        }
        false
    }

    pub fn draw(&self, camera_x: f32, camera_y: f32) {
        // Converter coordenadas do mundo para tela
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;
        
        // Desenhar checkpoint como uma bandeira ou poste
        if self.activated {
            // Checkpoint ativado - verde/escuro
            draw_rectangle(screen_x, screen_y, self.width, self.height, DARKGREEN);
            // Bandeira no topo
            draw_rectangle(screen_x + self.width - 10.0, screen_y, 10.0, 20.0, GREEN);
        } else {
            // Checkpoint não ativado - cinza
            draw_rectangle(screen_x, screen_y, self.width, self.height, GRAY);
            // Bandeira no topo
            draw_rectangle(screen_x + self.width - 10.0, screen_y, 10.0, 20.0, DARKGRAY);
        }
    }
}

pub fn create_level_checkpoints(level: usize) -> Vec<Checkpoint> {
    let mut checkpoints = Vec::new();
    
    // Chão está em y=550, então checkpoint deve estar em y=550-60=490 para ficar no chão
    // Para plataformas, usar y da plataforma - altura do checkpoint
    
    match level {
        1 => {
            // Fase 1 - Checkpoints estratégicos no chão
            checkpoints.push(Checkpoint::new(500.0, 490.0));  // Após primeira plataforma (no chão)
            checkpoints.push(Checkpoint::new(1200.0, 490.0)); // Meio da fase (no chão)
            checkpoints.push(Checkpoint::new(2000.0, 490.0));  // Próximo do fim (no chão)
        }
        2 => {
            // Fase 2 - No chão
            checkpoints.push(Checkpoint::new(600.0, 490.0));
            checkpoints.push(Checkpoint::new(1400.0, 490.0));
            checkpoints.push(Checkpoint::new(2200.0, 490.0));
        }
        3 => {
            // Fase 3 - No chão
            checkpoints.push(Checkpoint::new(700.0, 490.0));
            checkpoints.push(Checkpoint::new(1600.0, 490.0));
            checkpoints.push(Checkpoint::new(2400.0, 490.0));
        }
        4 => {
            // Fase 4 - No chão
            checkpoints.push(Checkpoint::new(800.0, 490.0));
            checkpoints.push(Checkpoint::new(1800.0, 490.0));
            checkpoints.push(Checkpoint::new(2600.0, 490.0));
        }
        5 => {
            checkpoints.push(Checkpoint::new(1400.0, 490.0));
            checkpoints.push(Checkpoint::new(3000.0, 490.0));
        }
        _ => {}
    }
    
    checkpoints
}

