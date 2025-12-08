use super::*;

impl Game {
    fn draw_level_world(&self) {
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
    fn draw_level_hud(&self, include_time_label: bool) {
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
    pub fn draw(&self) {
        clear_background(WHITE);
        match self.state {
            GameState::Splash => {
                let progress = (self.splash_timer / SPLASH_DURATION).min(1.0);
                let fade_in = if progress < 0.5 {
                    progress * 2.0
                } else if progress > 0.7 {
                    1.0 - ((progress - 0.7) / 0.3)
                } else {
                    1.0
                };
                let title = "JUMP QUEST";
                let title_size = MENU_TITLE_SIZE * 1.2;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                let title_color = Color::new(0.0, 0.0, 0.0, fade_in);
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    screen_height() / 2.0,
                    title_size,
                    title_color,
                );
                let version_text = format!("v{}", GAME_VERSION);
                let version_size = MENU_VERSION_SIZE;
                let version_width =
                    measure_text(&version_text, None, version_size as u16, 1.0).width;
                let version_color = Color::new(0.5, 0.5, 0.5, fade_in * 0.7);
                draw_text(
                    &version_text,
                    screen_width() / 2.0 - version_width / 2.0,
                    screen_height() / 2.0 + 80.0,
                    version_size,
                    version_color,
                );
            }
            GameState::Menu => {
                let title = "JUMP QUEST";
                let title_width = measure_text(title, None, MENU_TITLE_SIZE as u16, 1.0).width;
                let title_color = if self.is_easter_egg() {
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
                let menu_options = vec![
                    "CONTINUE", "PLAY", "CO-OP", "VERSUS", "SETTINGS", "CREDITS", "EXIT",
                ];
                let start_y = screen_height() / 2.0 - 40.0;
                for (i, option) in menu_options.iter().enumerate() {
                    let option_width =
                        measure_text(option, None, MENU_OPTION_SIZE as u16, 1.0).width;
                    let x = screen_width() / 2.0 - option_width / 2.0;
                    let y = start_y + (i as f32 * MENU_OPTION_SPACING);
                    let scale = self.menu_animation.get_scale(i);
                    let color = if i == self.menu_selection {
                        BLACK
                    } else {
                        DARKGRAY
                    };
                    let title_color = if self.is_easter_egg() {
                        Color::new(0.85, 0.65, 0.13, 1.0)
                    } else {
                        BLACK
                    };
                    if i == 0 && self.is_easter_egg() {
                        draw_text(
                            title,
                            screen_width() / 2.0 - title_width / 2.0,
                            screen_height() / 2.0 - 180.0,
                            MENU_TITLE_SIZE,
                            title_color,
                        );
                    }
                    if i == self.menu_selection {
                        let anim_time = self.menu_animation_time % (2.0 * std::f32::consts::PI);
                        let alpha = (anim_time.sin() * 0.4 + 0.6).clamp(0.3, 1.0);
                        let indicator_color = Color::new(0.0, 0.0, 0.0, alpha);
                        draw_text(
                            ">",
                            x - MENU_INDICATOR_OFFSET,
                            y,
                            MENU_OPTION_SIZE,
                            indicator_color,
                        );
                        draw_line(
                            x - 10.0,
                            y + 8.0,
                            x + option_width + 10.0,
                            y + 8.0,
                            2.0,
                            Color::new(0.0, 0.0, 0.0, alpha * 0.5),
                        );
                    }
                    let scaled_size = MENU_OPTION_SIZE * scale;
                    let scaled_width = measure_text(option, None, scaled_size as u16, 1.0).width;
                    draw_text(
                        option,
                        x + (option_width - scaled_width) / 2.0,
                        y - (scaled_size - MENU_OPTION_SIZE) / 2.0,
                        scaled_size,
                        color,
                    );
                    if i == 0 && self.has_new_save {
                        let new_text = "NEW";
                        draw_text(new_text, x + option_width + 10.0, y, 20.0, RED);
                    }
                    if i == 2 && !self.versus_played {
                        let new_text = "NEW";
                        draw_text(new_text, x + option_width + 10.0, y, 20.0, GREEN);
                    }
                }
                let last_option_y =
                    start_y + ((menu_options.len() - 1) as f32 * MENU_OPTION_SPACING);
                let last_option_height = MENU_OPTION_SIZE;
                let instructions_y = last_option_y + last_option_height + 40.0;
                let instructions = "ARROWS/WASD: Navigate | ENTER/SPACE: Select";
                let inst_width =
                    measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    instructions_y,
                    MENU_INSTRUCTION_SIZE,
                    GRAY,
                );
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
                let input_label = "Name:";
                let input_size = 32.0;
                draw_text(
                    input_label,
                    screen_width() / 2.0 - 200.0,
                    screen_height() / 2.0 - 50.0,
                    input_size,
                    BLACK,
                );
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
                let instructions = "ENTER: Confirm | ESC: Cancel";
                let inst_width =
                    measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
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
                        BLACK
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
                let page_text = format!("{}/{}", self.tutorial_page + 1, TUTORIAL_PAGE_COUNT);
                let page_width = measure_text(&page_text, None, 24, 1.0).width;
                draw_text(
                    &page_text,
                    screen_width() / 2.0 - page_width / 2.0,
                    screen_height() - 100.0,
                    24.0,
                    GRAY,
                );
                let instructions = "LEFT/RIGHT: Navigate | ENTER: Next/Start | ESC: Skip";
                let inst_width =
                    measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 40.0,
                    MENU_INSTRUCTION_SIZE,
                    GRAY,
                );
            }
            GameState::ContinueMenu => {
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
                    let color = if slot_idx == self.continue_selection {
                        BLACK
                    } else {
                        DARKGRAY
                    };
                    if slot_idx == self.continue_selection {
                        draw_text(">", 100.0, y, 32.0, BLACK);
                    }
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
                            if save_data.player_name.is_empty() {
                                "Unknown"
                            } else {
                                &save_data.player_name
                            }
                        );
                        draw_text(&slot_info, 150.0, y, 24.0, color);
                    } else {
                        let empty_text = format!("Slot {}: Empty Slot", slot_num);
                        draw_text(&empty_text, 150.0, y, 24.0, color);
                    }
                }
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
                    let instructions = "ENTER: Load | DELETE: Erase | ESC: Back";
                    let inst_width =
                        measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
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
                let confirm_text = "ARE YOU SURE YOU WANT TO EXIT?";
                let confirm_size = 42.0;
                let confirm_width =
                    measure_text(confirm_text, None, confirm_size as u16, 1.0).width;
                draw_text(
                    confirm_text,
                    screen_width() / 2.0 - confirm_width / 2.0,
                    screen_height() / 2.0 - 100.0,
                    confirm_size,
                    BLACK,
                );
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
                let option_size = 30.0;
                let start_y = 180.0;
                let spacing = 60.0;
                let safe_resolution_index = self
                    .resolution_index
                    .min(self.available_resolutions.len().saturating_sub(1));
                let res_name = if safe_resolution_index < self.available_resolutions.len() {
                    let (w, h) = self.available_resolutions[safe_resolution_index];
                    format!("{}x{}", w, h)
                } else {
                    "Unknown".to_string()
                };
                let sound_text =
                    format!("SOUND: {}", if self.sound_enabled { "ON" } else { "OFF" });
                let sound_color = if self.settings_selection == 0 {
                    BLACK
                } else {
                    GRAY
                };
                let sound_width = measure_text(&sound_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 0 {
                    draw_text(
                        ">",
                        screen_width() / 2.0 - sound_width / 2.0 - 30.0,
                        start_y,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        "<",
                        screen_width() / 2.0 + sound_width / 2.0 + 10.0,
                        start_y,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &sound_text,
                    screen_width() / 2.0 - sound_width / 2.0,
                    start_y,
                    option_size,
                    sound_color,
                );
                let res_text = format!("RESOLUTION: {}", res_name);
                let res_color = if self.settings_selection == 1 {
                    BLACK
                } else {
                    GRAY
                };
                let res_width = measure_text(&res_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 1 {
                    draw_text(
                        "<",
                        screen_width() / 2.0 - res_width / 2.0 - 30.0,
                        start_y + spacing,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        ">",
                        screen_width() / 2.0 + res_width / 2.0 + 10.0,
                        start_y + spacing,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &res_text,
                    screen_width() / 2.0 - res_width / 2.0,
                    start_y + spacing,
                    option_size,
                    res_color,
                );
                let controls_text = "CONTROLS";
                let controls_color = if self.settings_selection == 2 {
                    BLACK
                } else {
                    GRAY
                };
                let controls_width =
                    measure_text(controls_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 2 {
                    draw_text(
                        ">",
                        screen_width() / 2.0 - controls_width / 2.0 - 30.0,
                        start_y + spacing * 2.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    controls_text,
                    screen_width() / 2.0 - controls_width / 2.0,
                    start_y + spacing * 2.0,
                    option_size,
                    controls_color,
                );
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
                let difficulty_name = match self.difficulty_multiplier {
                    x if x == DIFFICULTY_EASY => "EASY",
                    x if x == DIFFICULTY_NORMAL => "NORMAL",
                    x if x == DIFFICULTY_HARD => "HARD",
                    x if x == DIFFICULTY_INSANE => "INSANE",
                    _ => "NORMAL",
                };
                let difficulty_text = format!("DIFFICULTY: {}", difficulty_name);
                let difficulty_color = if self.settings_selection == 3 {
                    BLACK
                } else {
                    GRAY
                };
                let difficulty_width =
                    measure_text(&difficulty_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 3 {
                    draw_text(
                        "<",
                        screen_width() / 2.0 - difficulty_width / 2.0 - 30.0,
                        start_y + spacing * 3.0,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        ">",
                        screen_width() / 2.0 + difficulty_width / 2.0 + 10.0,
                        start_y + spacing * 3.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &difficulty_text,
                    screen_width() / 2.0 - difficulty_width / 2.0,
                    start_y + spacing * 3.0,
                    option_size,
                    difficulty_color,
                );
                let colorblind_text = format!(
                    "COLORBLIND MODE: {}",
                    if self.colorblind_mode { "ON" } else { "OFF" }
                );
                let colorblind_color = if self.settings_selection == 4 {
                    BLACK
                } else {
                    GRAY
                };
                let colorblind_width =
                    measure_text(&colorblind_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 4 {
                    draw_text(
                        ">",
                        screen_width() / 2.0 - colorblind_width / 2.0 - 30.0,
                        start_y + spacing * 4.0,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        "<",
                        screen_width() / 2.0 + colorblind_width / 2.0 + 10.0,
                        start_y + spacing * 4.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &colorblind_text,
                    screen_width() / 2.0 - colorblind_width / 2.0,
                    start_y + spacing * 4.0,
                    option_size,
                    colorblind_color,
                );
                let font_text = format!("FONT SIZE: {:.0}%", self.font_size_scale * 100.0);
                let font_color = if self.settings_selection == 5 {
                    BLACK
                } else {
                    GRAY
                };
                let font_width = measure_text(&font_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 5 {
                    draw_text(
                        "<",
                        screen_width() / 2.0 - font_width / 2.0 - 30.0,
                        start_y + spacing * 5.0,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        ">",
                        screen_width() / 2.0 + font_width / 2.0 + 10.0,
                        start_y + spacing * 5.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &font_text,
                    screen_width() / 2.0 - font_width / 2.0,
                    start_y + spacing * 5.0,
                    option_size,
                    font_color,
                );
                let assist_text = format!(
                    "ASSIST MODE: {}",
                    if self.assist_mode { "ON" } else { "OFF" }
                );
                let assist_color = if self.settings_selection == 6 {
                    BLACK
                } else {
                    GRAY
                };
                let assist_width = measure_text(&assist_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 6 {
                    draw_text(
                        ">",
                        screen_width() / 2.0 - assist_width / 2.0 - 30.0,
                        start_y + spacing * 6.0,
                        option_size,
                        BLACK,
                    );
                    draw_text(
                        "<",
                        screen_width() / 2.0 + assist_width / 2.0 + 10.0,
                        start_y + spacing * 6.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    &assist_text,
                    screen_width() / 2.0 - assist_width / 2.0,
                    start_y + spacing * 6.0,
                    option_size,
                    assist_color,
                );
                let back_text = "BACK";
                let back_color = if self.settings_selection == 7 {
                    BLACK
                } else {
                    GRAY
                };
                let back_width = measure_text(back_text, None, option_size as u16, 1.0).width;
                if self.settings_selection == 7 {
                    draw_text(
                        ">",
                        screen_width() / 2.0 - back_width / 2.0 - 30.0,
                        start_y + spacing * 7.0,
                        option_size,
                        BLACK,
                    );
                }
                draw_text(
                    back_text,
                    screen_width() / 2.0 - back_width / 2.0,
                    start_y + spacing * 7.0,
                    option_size,
                    back_color,
                );
                let instructions =
                    "Use ARROWS to navigate and adjust, ENTER to confirm, ESC to go back";
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
            GameState::Controls => {
                let title = "CONTROLS";
                let title_size = 48.0;
                let title_width = measure_text(title, None, title_size as u16, 1.0).width;
                draw_text(
                    title,
                    screen_width() / 2.0 - title_width / 2.0,
                    80.0,
                    title_size,
                    BLACK,
                );
                if let Some((player, action)) = self.controls_waiting_input {
                    let waiting_text = format!(
                        "Press key for Player {} {}...",
                        player,
                        match action {
                            ControlAction::Left => "LEFT",
                            ControlAction::Right => "RIGHT",
                            ControlAction::Jump => "JUMP",
                        }
                    );
                    let waiting_width = measure_text(&waiting_text, None, 32, 1.0).width;
                    draw_text(
                        &waiting_text,
                        screen_width() / 2.0 - waiting_width / 2.0,
                        screen_height() / 2.0,
                        32.0,
                        RED,
                    );
                    let esc_text = "Press ESC to cancel";
                    let esc_width = measure_text(esc_text, None, 20, 1.0).width;
                    draw_text(
                        esc_text,
                        screen_width() / 2.0 - esc_width / 2.0,
                        screen_height() / 2.0 + 50.0,
                        20.0,
                        GRAY,
                    );
                } else {
                    let option_size = 28.0;
                    let start_y = 180.0;
                    let spacing = 50.0;
                    let player_text = format!("PLAYER {}", self.controls_player);
                    let player_color = BLACK;
                    let player_width = measure_text(&player_text, None, 36, 1.0).width;
                    draw_text(
                        &player_text,
                        screen_width() / 2.0 - player_width / 2.0,
                        start_y,
                        36.0,
                        player_color,
                    );
                    let controls = if self.controls_player == 1 {
                        &self.player1_controls
                    } else {
                        &self.player2_controls
                    };
                    let actions = [
                        ("LEFT", controls.left, controls.left_gamepad),
                        ("RIGHT", controls.right, controls.right_gamepad),
                        ("JUMP", controls.jump, controls.jump_gamepad),
                    ];
                    for (i, (action_name, key, gamepad)) in actions.iter().enumerate() {
                        let y = start_y + 80.0 + (i as f32 * spacing);
                        let color = if i == self.controls_selection {
                            BLACK
                        } else {
                            GRAY
                        };
                        let key_name = key
                            .map(|k| format!("{:?}", k))
                            .unwrap_or_else(|| "None".to_string());
                        let gamepad_text = gamepad
                            .map(|g| format!("GP{}", g))
                            .unwrap_or_else(|| "".to_string());
                        let binding_text = if !gamepad_text.is_empty() {
                            format!("{}: {} / {}", action_name, key_name, gamepad_text)
                        } else {
                            format!("{}: {}", action_name, key_name)
                        };
                        if i == self.controls_selection {
                            draw_text(">", screen_width() / 2.0 - 200.0, y, option_size, BLACK);
                        }
                        let binding_width =
                            measure_text(&binding_text, None, option_size as u16, 1.0).width;
                        draw_text(
                            &binding_text,
                            screen_width() / 2.0 - binding_width / 2.0,
                            y,
                            option_size,
                            color,
                        );
                    }
                    let instructions = "UP/DOWN: Select Player | LEFT/RIGHT: Select Action | ENTER: Rebind | ESC: Back";
                    let inst_width = measure_text(instructions, None, 16, 1.0).width;
                    draw_text(
                        instructions,
                        screen_width() / 2.0 - inst_width / 2.0,
                        screen_height() - 40.0,
                        16.0,
                        GRAY,
                    );
                }
            }
            GameState::Credits => {
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
                let start_y = 140.0;
                let spacing = 28.0;
                let section_spacing = 40.0;
                let mut current_y = start_y;
                let game_title = "JUMP QUEST";
                let game_title_size = 36.0;
                let game_title_width =
                    measure_text(game_title, None, game_title_size as u16, 1.0).width;
                draw_text(
                    game_title,
                    screen_width() / 2.0 - game_title_width / 2.0,
                    current_y,
                    game_title_size,
                    BLACK,
                );
                current_y += section_spacing;
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
                let developer = "guicybercode";
                let developer_size = 32.0;
                let developer_width =
                    measure_text(developer, None, developer_size as u16, 1.0).width;
                draw_text(
                    developer,
                    screen_width() / 2.0 - developer_width / 2.0,
                    current_y,
                    developer_size,
                    BLACK,
                );
                current_y += section_spacing;
                let stack_title = "Technology Stack";
                let stack_title_size = 24.0;
                let stack_title_width =
                    measure_text(stack_title, None, stack_title_size as u16, 1.0).width;
                draw_text(
                    stack_title,
                    screen_width() / 2.0 - stack_title_width / 2.0,
                    current_y,
                    stack_title_size,
                    BLACK,
                );
                current_y += spacing;
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
                let info_title = "Game Information";
                let info_title_size = 24.0;
                let info_title_width =
                    measure_text(info_title, None, info_title_size as u16, 1.0).width;
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
                self.draw_level_world();
                self.draw_level_hud(false);
                self.draw_level_start_fade();
            }
            GameState::Versus => {
                let camera_x = self.camera.x;
                let camera_y = self.camera.y;
                for platform in &self.versus_platforms {
                    platform.draw(camera_x, camera_y);
                }
                if self.respawn_timer_p1 <= 0.0 {
                    self.player.draw_vs(camera_x, camera_y, true);
                }
                if let Some(ref p2) = self.player2 {
                    if self.respawn_timer_p2 <= 0.0 {
                        p2.draw_vs(camera_x, camera_y, false);
                    }
                }
                let p1_score_text = format!(
                    "P1: {} kills | {} pts",
                    self.player1_score, self.player1_points
                );
                let p2_score_text = format!(
                    "P2: {} kills | {} pts",
                    self.player2_score, self.player2_points
                );
                draw_text(&p1_score_text, 20.0, 30.0, 24.0, BLACK);
                let p2_width = measure_text(&p2_score_text, None, 24u16, 1.0).width;
                draw_text(
                    &p2_score_text,
                    screen_width() - p2_width - 20.0,
                    30.0,
                    24.0,
                    DARKGRAY,
                );
                let time_text = format!("{}", self.versus_time_remaining as u32);
                let time_width = measure_text(&time_text, None, 28u16, 1.0).width;
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
                let instructions = "P1: WASD | P2: Arrow Keys | ESC: Menu";
                let inst_width = measure_text(instructions, None, 16u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 30.0,
                    16.0,
                    GRAY,
                );
                self.draw_level_start_fade();
            }
            GameState::Coop => {
                self.draw_level_world();
                if let Some(ref p2) = self.player2 {
                    p2.draw(self.camera.x, self.camera.y);
                }
                self.draw_level_hud(true);
                let p1_text = "P1";
                let p2_text = "P2";
                draw_text(p1_text, 10.0, 10.0, 20.0, BLUE);
                if let Some(ref _p2) = self.player2 {
                    let p2_width = measure_text(p2_text, None, 20u16, 1.0).width;
                    draw_text(p2_text, screen_width() - p2_width - 10.0, 10.0, 20.0, RED);
                }
                let instructions = "P1: WASD | P2: Arrow Keys | P: Pause | ESC: Menu";
                let inst_width = measure_text(instructions, None, 16u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 30.0,
                    16.0,
                    GRAY,
                );
                self.draw_level_start_fade();
            }
            GameState::VersusEnd => {
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
                let score_size = 28.0;
                let p1_final_text = format!(
                    "Player 1: {} kills | {} points",
                    self.player1_score, self.player1_points
                );
                let p2_final_text = format!(
                    "Player 2: {} kills | {} points",
                    self.player2_score, self.player2_points
                );
                let p1_final_width =
                    measure_text(&p1_final_text, None, score_size as u16, 1.0).width;
                let p2_final_width =
                    measure_text(&p2_final_text, None, score_size as u16, 1.0).width;
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
                let spacing = 100.0;
                let level_count = self.unlocked_levels.len();
                let start_x = screen_width() / 2.0 - (spacing * (level_count as f32 - 1.0)) / 2.0;
                let center_y = screen_height() / 2.0;
                let level_names: Vec<String> =
                    (1..=level_count).map(|i| format!("Level {}", i)).collect();
                for i in 0..level_count {
                    let x = start_x + (i as f32 * spacing);
                    let is_selected = i == self.level_selection;
                    let is_unlocked = self.unlocked_levels[i];
                    let circle_color = if !is_unlocked {
                        DARKGRAY
                    } else if is_selected {
                        BLACK
                    } else {
                        GRAY
                    };
                    let circle_radius = if is_selected { 35.0 } else { 30.0 };
                    draw_circle(x, center_y, circle_radius, circle_color);
                    draw_circle_lines(
                        x,
                        center_y,
                        circle_radius,
                        3.0,
                        if is_selected { WHITE } else { BLACK },
                    );
                    let num_text = format!("{}", i + 1);
                    let num_size = 36.0;
                    let num_width = measure_text(&num_text, None, num_size as u16, 1.0).width;
                    draw_text(
                        &num_text,
                        x - num_width / 2.0,
                        center_y + num_size / 3.0,
                        num_size,
                        if is_unlocked {
                            if is_selected {
                                WHITE
                            } else {
                                BLACK
                            }
                        } else {
                            GRAY
                        },
                    );
                    let (difficulty, coin_count, difficulty_color) = self.get_level_info(i + 1);
                    if is_unlocked {
                        let name_size = 18.0;
                        let name_text = &level_names[i];
                        let name_width = measure_text(name_text, None, name_size as u16, 1.0).width;
                        draw_text(
                            name_text,
                            x - name_width / 2.0,
                            center_y + 60.0,
                            name_size,
                            if is_selected { BLACK } else { GRAY },
                        );
                        let diff_text = difficulty;
                        let diff_width = measure_text(&diff_text, None, 16, 1.0).width;
                        draw_text(
                            &diff_text,
                            x - diff_width / 2.0,
                            center_y + 82.0,
                            16.0,
                            difficulty_color,
                        );
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
                self.draw_level_world();
                self.draw_level_hud(true);
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, 0.6),
                );
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
                    let option_width =
                        measure_text(option, None, MENU_OPTION_SIZE as u16, 1.0).width;
                    let x = screen_width() / 2.0 - option_width / 2.0;
                    let y = start_y + (i as f32 * MENU_OPTION_SPACING);
                    let scale = self.pause_animation.get_scale(i);
                    let color = if i == self.pause_selection {
                        WHITE
                    } else {
                        LIGHTGRAY
                    };
                    if i == self.pause_selection {
                        draw_text(">", x - MENU_INDICATOR_OFFSET, y, MENU_OPTION_SIZE, WHITE);
                    }
                    let scaled_size = MENU_OPTION_SIZE * scale;
                    let scaled_width = measure_text(option, None, scaled_size as u16, 1.0).width;
                    draw_text(
                        option,
                        x + (option_width - scaled_width) / 2.0,
                        y - (scaled_size - MENU_OPTION_SIZE) / 2.0,
                        scaled_size,
                        color,
                    );
                }
                let instructions = "ARROWS/WASD: Navigate | ENTER: Select | P/ESC: Resume";
                let inst_width =
                    measure_text(instructions, None, MENU_INSTRUCTION_SIZE as u16, 1.0).width;
                draw_text(
                    instructions,
                    screen_width() / 2.0 - inst_width / 2.0,
                    screen_height() - 40.0,
                    MENU_INSTRUCTION_SIZE,
                    LIGHTGRAY,
                );
            }
            GameState::Respawn => {
                let time_remaining = self.respawn_timer;
                let total_time = 3.0;
                let progress = 1.0 - (time_remaining / total_time);
                let bg_alpha = (progress * 0.8).min(0.8);
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, bg_alpha),
                );
                let player_size = 64.0;
                let player_x = screen_width() / 2.0 - player_size / 2.0;
                let player_y = screen_height() / 2.0 - player_size / 2.0 - 50.0;
                let player_alpha = (1.0 - progress * 1.5).max(0.0);
                let player_color = Color::new(0.0, 0.0, 0.0, player_alpha);
                draw_rectangle(player_x, player_y, player_size, player_size, player_color);
                draw_rectangle_lines(
                    player_x,
                    player_y,
                    player_size,
                    player_size,
                    2.0,
                    Color::new(1.0, 1.0, 1.0, player_alpha),
                );
                let eye_size = 6.0;
                let eye_y = player_y + 20.0;
                draw_circle(
                    player_x + 20.0,
                    eye_y,
                    eye_size,
                    Color::new(1.0, 1.0, 1.0, player_alpha),
                );
                draw_circle(
                    player_x + 44.0,
                    eye_y,
                    eye_size,
                    Color::new(1.0, 1.0, 1.0, player_alpha),
                );
                let lives_text = format!("Lives: {}", self.lives);
                let lives_size = 48.0;
                let lives_width = measure_text(&lives_text, None, lives_size as u16, 1.0).width;
                let lives_alpha = 1.0 - (progress * 0.5).min(0.5);
                let lives_color = Color::new(1.0, 1.0, 1.0, lives_alpha);
                draw_text(
                    &lives_text,
                    screen_width() / 2.0 - lives_width / 2.0,
                    screen_height() / 2.0 + 80.0,
                    lives_size,
                    lives_color,
                );
                let countdown = (time_remaining.ceil() as u32).max(1);
                let countdown_text = format!("Respawn in {}...", countdown);
                let countdown_size = 32.0;
                let countdown_width =
                    measure_text(&countdown_text, None, countdown_size as u16, 1.0).width;
                draw_text(
                    &countdown_text,
                    screen_width() / 2.0 - countdown_width / 2.0,
                    screen_height() / 2.0 + 140.0,
                    countdown_size,
                    lives_color,
                );
            }
            GameState::GameOver => {
                let fade_progress = if self.game_over_fade_timer > 0.0 {
                    (2.0 - self.game_over_fade_timer) / 2.0
                } else {
                    1.0
                };
                let eased_progress = fade_progress * fade_progress * (3.0 - 2.0 * fade_progress);
                let fade_alpha = eased_progress.min(1.0);
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, fade_alpha),
                );
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
                    let coins_text =
                        format!("Coins: {}/{}", self.coins_collected, self.total_coins);
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
                    let restart_width =
                        measure_text(restart_text, None, restart_size as u16, 1.0).width;
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
                if self.current_level < MAX_LEVELS
                    && self.current_level < self.unlocked_levels.len()
                    && self.unlocked_levels[self.current_level]
                {
                    let unlock_text = format!("Level {} unlocked!", self.current_level + 1);
                    let unlock_size = 24.0;
                    let unlock_width =
                        measure_text(&unlock_text, None, unlock_size as u16, 1.0).width;
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
                let continue_width =
                    measure_text(continue_text, None, continue_size as u16, 1.0).width;
                draw_text(
                    continue_text,
                    screen_width() / 2.0 - continue_width / 2.0,
                    screen_height() / 2.0 + 60.0,
                    continue_size,
                    GRAY,
                );
            }
        }
        self.draw_transition();
        self.draw_error_message();
    }
    fn draw_error_message(&self) {
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
