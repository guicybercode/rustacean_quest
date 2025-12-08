use crate::constants::*;
use macroquad::prelude::*;

use super::Game;

impl Game {
    pub fn draw_level_world(&self) {
        let screen_left = self.camera.x - COLLISION_MARGIN;
        let screen_right = self.camera.x + screen_width() + COLLISION_MARGIN;
        let screen_top = self.camera.y - COLLISION_MARGIN;
        let screen_bottom = self.camera.y + screen_height() + COLLISION_MARGIN;
        for platform in &self.platforms {
            if platform.x + platform.width >= screen_left
                && platform.x <= screen_right
                && platform.y + platform.height >= screen_top
                && platform.y <= screen_bottom
            {
                platform.draw(self.camera.x, self.camera.y);
            }
        }
        for checkpoint in &self.checkpoints {
            if checkpoint.x >= screen_left
                && checkpoint.x <= screen_right
                && checkpoint.y >= screen_top
                && checkpoint.y <= screen_bottom
            {
                checkpoint.draw(self.camera.x, self.camera.y);
            }
        }
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
        self.player.draw(self.camera.x, self.camera.y);
        for particle in &self.particles {
            particle.draw(self.camera.x, self.camera.y, self.colorblind_mode);
        }
        for bounce in &self.coin_bounces {
            bounce.draw(self.camera.x, self.camera.y, self.colorblind_mode);
        }
    }

    pub fn draw_level_hud(&self, include_time_label: bool) {
        let font_scale = self.font_size_scale;
        let time_seconds = self.time_remaining as u32;
        let time_text = if include_time_label {
            format!("Time: {}s", time_seconds)
        } else {
            format!("{}", time_seconds)
        };
        let time_color = if self.colorblind_mode {
            if self.time_remaining < TIME_WARNING_RED {
                DARKGRAY
            } else if self.time_remaining < TIME_WARNING_YELLOW {
                GRAY
            } else {
                BLACK
            }
        } else {
            if self.time_remaining < TIME_WARNING_RED {
                RED
            } else if self.time_remaining < TIME_WARNING_YELLOW {
                YELLOW
            } else {
                BLACK
            }
        };
        let player_name_display = if self.player_name.is_empty() {
            "Player"
        } else {
            &self.player_name
        };
        draw_text(player_name_display, 10.0, 30.0, 24.0 * font_scale, BLACK);
        draw_text(
            &format!(
                "Level: {} | Coins: {}/{} | Time: {}s",
                self.current_level, self.coins_collected, self.total_coins, time_seconds
            ),
            10.0,
            60.0,
            30.0 * font_scale,
            BLACK,
        );
        let score_text = format!("Score: {}", self.score);
        draw_text(&score_text, 10.0, 100.0, 28.0 * font_scale, BLACK);
        let lives_text = format!("Lives: {}", self.lives);
        let lives_color = if self.colorblind_mode {
            if self.lives <= 1 {
                DARKGRAY
            } else {
                BLACK
            }
        } else {
            if self.lives <= 1 {
                RED
            } else {
                BLACK
            }
        };
        draw_text(&lives_text, 10.0, 130.0, 28.0 * font_scale, lives_color);
        let time_width = measure_text(&time_text, None, (40.0 * font_scale) as u16, 1.0).width;
        draw_text(
            &time_text,
            screen_width() - time_width - 20.0,
            40.0,
            40.0 * font_scale,
            time_color,
        );
    }

    pub fn draw_error_message(&self) {
        if let Some(ref error) = self.error_message {
            let alpha = (self.error_timer / 5.0).min(1.0);
            let bg_color = Color::new(0.0, 0.0, 0.0, alpha * 0.8);
            let text_color = Color::new(1.0, 0.3, 0.3, alpha);
            let padding = 20.0;
            let font_size = 24.0;
            let text_width = measure_text(error, None, font_size as u16, 1.0).width;
            let box_width = text_width + padding * 2.0;
            let box_height = font_size + padding * 2.0;
            let x = screen_width() / 2.0 - box_width / 2.0;
            let y = screen_height() - box_height - 50.0;
            draw_rectangle(x, y, box_width, box_height, bg_color);
            draw_rectangle_lines(x, y, box_width, box_height, 2.0, text_color);
            draw_text(
                error,
                x + padding,
                y + padding + font_size,
                font_size,
                text_color,
            );
        }
    }
}

