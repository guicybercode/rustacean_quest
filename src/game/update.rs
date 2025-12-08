use super::*;

impl Game {
    pub fn update(&mut self, dt: f32) {
        self.update_transition(dt);

        if self.error_timer > 0.0 {
            self.error_timer -= dt;
            if self.error_timer <= 0.0 {
                self.error_message = None;
            }
        }
        self.particles.iter_mut().for_each(|p| p.update(dt));
        self.particles.retain(|p| p.is_alive());
        let max_particles = PARTICLE_COUNT * 20;
        if self.particles.len() > max_particles {
            let excess = self.particles.len() - max_particles;
            self.particles.drain(0..excess);
        }
        self.coin_bounces.iter_mut().for_each(|b| b.update(dt));
        self.coin_bounces.retain(|b| b.is_alive());
        self.menu_animation.update(dt);
        self.pause_animation.update(dt);
        self.camera_shake.update(dt);

        if matches!(self.state, GameState::Menu) {
            self.save_check_timer += dt;
            if self.save_check_timer >= SAVE_CHECK_INTERVAL {
                self.check_for_new_saves();
                self.save_check_timer = 0.0;
            }
        } else {
            self.save_check_timer = 0.0;
        }

        if self.transition.is_active() && !matches!(self.state, GameState::Menu) {
            return;
        }

        match self.state {
            GameState::Splash => {
                self.splash_timer += dt;
                if self.splash_timer >= SPLASH_DURATION {
                    self.state = GameState::Menu;
                    self.splash_timer = 0.0;
                    self.splash_shown = true;
                }
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Escape)
                {
                    self.state = GameState::Menu;
                    self.splash_timer = 0.0;
                    self.splash_shown = true;
                }
            }
            GameState::Menu => {
                self.menu_animation_time += dt * MENU_ANIMATION_SPEED;
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if self.menu_selection > 0 {
                        self.menu_selection -= 1;
                        self.menu_animation_time = 0.0;
                        self.menu_animation.trigger(self.menu_selection);
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if self.menu_selection < 6 {
                        self.menu_selection += 1;
                        self.menu_animation_time = 0.0;
                        self.menu_animation.trigger(self.menu_selection);
                        self.audio.play_menu_select();
                    }
                }
                if self.previous_menu_selection != self.menu_selection {
                    self.previous_menu_selection = self.menu_selection;
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.audio.play_menu_select();
                    match self.menu_selection {
                        0 => {
                            self.has_new_save = false;
                            self.start_transition(GameState::ContinueMenu);
                            self.continue_selection = 0;
                            self.continue_mode = ContinueMode::View;
                        }
                        1 => {
                            self.start_transition(GameState::NameInput);
                            self.name_input.clear();
                            self.name_input_error = None;
                        }
                        2 => {
                            self.start_transition(GameState::LevelSelect);
                            self.level_selection = 0;
                        }
                        3 => {
                            self.versus_played = true;
                            self.load_versus_map();
                            self.start_transition(GameState::Versus);
                        }
                        4 => {
                            self.state = GameState::Settings;
                            self.settings_selection = 0;
                        }
                        5 => {
                            self.state = GameState::Credits;
                        }
                        6 => {
                            self.state = GameState::MenuExitConfirm;
                            self.menu_selection = 0;
                        }
                        _ => {}
                    }
                }
            }
            GameState::MenuExitConfirm => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace) {
                    self.audio.play_menu_select();
                    self.state = GameState::Menu;
                }
            }
            GameState::NameInput => {
                if let Some(ch) = get_char_pressed() {
                    if ch.is_alphanumeric() || ch == ' ' || ch == '-' || ch == '_' {
                        if self.name_input.len() < 20 {
                            self.name_input.push(ch);
                            self.name_input_error = None;
                        }
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    self.name_input.pop();
                    self.name_input_error = None;
                }
                let (is_valid, error_msg) = name_filter::is_name_valid(&self.name_input);
                if !is_valid && !self.name_input.is_empty() {
                    self.name_input_error = error_msg;
                } else {
                    self.name_input_error = None;
                }
                if is_key_pressed(KeyCode::Enter) {
                    let (is_valid, _) = name_filter::is_name_valid(&self.name_input);
                    if is_valid {
                        self.player_name = self.name_input.clone();
                        if self.player_name.to_lowercase() == "guicybercode" {
                            self.lives = EASTER_EGG_LIVES;
                        }
                        self.state = GameState::LevelSelect;
                        self.level_selection = 0;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.name_input.clear();
                    self.name_input_error = None;
                    self.state = GameState::Menu;
                }
            }
            GameState::ContinueMenu => {
                if self.continue_mode == ContinueMode::View {
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
                    if is_key_pressed(KeyCode::Enter) {
                        match SaveData::get_save_path(self.continue_selection) {
                            Ok(path) => {
                                if SaveData::save_exists(&path) {
                                    if let Err(e) = self.load_game(self.continue_selection) {
                                        let error_msg = format!("Error loading save: {}", e);
                                        eprintln!("{}", error_msg);
                                        self.show_error(error_msg);
                                    } else {
                                        self.load_level(
                                            self.current_level,
                                            self.last_checkpoint_pos.is_some(),
                                            Some(self.time_remaining),
                                            Some(self.coins_collected),
                                        );
                                        self.start_transition(GameState::Playing);
                                    }
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Error resolving save path: {}", e);
                                eprintln!("{}", error_msg);
                                self.show_error(error_msg);
                            }
                        }
                    }
                    if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
                        match SaveData::get_save_path(self.continue_selection) {
                            Ok(path) => {
                                if SaveData::save_exists(&path) {
                                    self.continue_mode = ContinueMode::DeleteConfirm;
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Error resolving save path: {}", e);
                                eprintln!("{}", error_msg);
                                self.show_error(error_msg);
                            }
                        }
                    }
                    if is_key_pressed(KeyCode::Escape) {
                        self.state = GameState::Menu;
                    }
                } else {
                    if is_key_pressed(KeyCode::Y) {
                        if let Err(e) = SaveData::delete_save(self.continue_selection) {
                            let error_msg = format!("Error deleting save: {}", e);
                            eprintln!("{}", error_msg);
                            self.show_error(error_msg);
                        }
                        self.continue_mode = ContinueMode::View;
                    }
                    if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
                        self.continue_mode = ContinueMode::View;
                    }
                }
            }
            GameState::Settings => {
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if self.settings_selection > 0 {
                        self.settings_selection -= 1;
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if self.settings_selection < 7 {
                        self.settings_selection += 1;
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    match self.settings_selection {
                        0 => {
                            self.sound_enabled = !self.sound_enabled;
                            self.audio.set_enabled(self.sound_enabled);
                            self.audio.play_menu_select();
                        }
                        1 => {
                            if self.resolution_index > 0 {
                                self.resolution_index -= 1;
                                self.apply_resolution();
                                self.audio.play_menu_select();
                            }
                        }
                        3 => {
                            self.difficulty_multiplier = match self.difficulty_multiplier {
                                x if x == DIFFICULTY_EASY => DIFFICULTY_NORMAL,
                                x if x == DIFFICULTY_NORMAL => DIFFICULTY_HARD,
                                x if x == DIFFICULTY_HARD => DIFFICULTY_INSANE,
                                _ => DIFFICULTY_EASY,
                            };
                            self.audio.play_menu_select();
                        }
                        4 => {
                            self.colorblind_mode = !self.colorblind_mode;
                            self.audio.play_menu_select();
                        }
                        5 => {
                            self.font_size_scale = match self.font_size_scale {
                                1.0 => 1.25,
                                1.25 => 1.5,
                                1.5 => 0.75,
                                _ => 1.0,
                            };
                            self.audio.play_menu_select();
                        }
                        6 => {
                            self.assist_mode = !self.assist_mode;
                            self.audio.play_menu_select();
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    match self.settings_selection {
                        0 => {
                            self.sound_enabled = !self.sound_enabled;
                            self.audio.set_enabled(self.sound_enabled);
                            self.audio.play_menu_select();
                        }
                        1 => {
                            if self.resolution_index < self.available_resolutions.len() - 1 {
                                self.resolution_index += 1;
                                self.apply_resolution();
                                self.audio.play_menu_select();
                            }
                        }
                        3 => {
                            self.difficulty_multiplier = match self.difficulty_multiplier {
                                x if x == DIFFICULTY_EASY => DIFFICULTY_NORMAL,
                                x if x == DIFFICULTY_NORMAL => DIFFICULTY_HARD,
                                x if x == DIFFICULTY_HARD => DIFFICULTY_INSANE,
                                _ => DIFFICULTY_EASY,
                            };
                            self.audio.play_menu_select();
                        }
                        4 => {
                            self.colorblind_mode = !self.colorblind_mode;
                            self.audio.play_menu_select();
                        }
                        5 => {
                            self.font_size_scale = match self.font_size_scale {
                                1.0 => 1.25,
                                1.25 => 1.5,
                                1.5 => 0.75,
                                _ => 1.0,
                            };
                            self.audio.play_menu_select();
                        }
                        6 => {
                            self.assist_mode = !self.assist_mode;
                            self.audio.play_menu_select();
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    match self.settings_selection {
                        0 => {
                            self.sound_enabled = !self.sound_enabled;
                            self.audio.set_enabled(self.sound_enabled);
                            self.audio.play_menu_select();
                        }
                        2 => {
                            self.state = GameState::Controls;
                            self.controls_selection = 0;
                            self.controls_player = 1;
                            self.controls_waiting_input = None;
                        }
                        7 => {
                            self.audio.play_menu_select();
                            if self.came_from_pause {
                                self.came_from_pause = false;
                                self.state = GameState::Pause;
                            } else {
                                self.state = GameState::Menu;
                                self.menu_selection = 0;
                            }
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.audio.play_menu_select();
                    if self.came_from_pause {
                        self.came_from_pause = false;
                        self.state = GameState::Pause;
                    } else {
                        self.state = GameState::Menu;
                        self.menu_selection = 0;
                    }
                }
            }
            GameState::Controls => {
                if let Some((player, action)) = self.controls_waiting_input {
                    let mut _captured = false;
                    let keys_to_check = [
                        KeyCode::A,
                        KeyCode::B,
                        KeyCode::C,
                        KeyCode::D,
                        KeyCode::E,
                        KeyCode::F,
                        KeyCode::G,
                        KeyCode::H,
                        KeyCode::I,
                        KeyCode::J,
                        KeyCode::K,
                        KeyCode::L,
                        KeyCode::M,
                        KeyCode::N,
                        KeyCode::O,
                        KeyCode::P,
                        KeyCode::Q,
                        KeyCode::R,
                        KeyCode::S,
                        KeyCode::T,
                        KeyCode::U,
                        KeyCode::V,
                        KeyCode::W,
                        KeyCode::X,
                        KeyCode::Y,
                        KeyCode::Z,
                        KeyCode::Space,
                        KeyCode::Enter,
                        KeyCode::Escape,
                        KeyCode::Left,
                        KeyCode::Right,
                        KeyCode::Up,
                        KeyCode::Down,
                        KeyCode::Key0,
                        KeyCode::Key1,
                        KeyCode::Key2,
                        KeyCode::Key3,
                        KeyCode::Key4,
                        KeyCode::Key5,
                        KeyCode::Key6,
                        KeyCode::Key7,
                        KeyCode::Key8,
                        KeyCode::Key9,
                    ];
                    for &keycode in &keys_to_check {
                        if is_key_pressed(keycode) {
                            let controls = if player == 1 {
                                &mut self.player1_controls
                            } else {
                                &mut self.player2_controls
                            };
                            match action {
                                ControlAction::Left => controls.left = Some(keycode),
                                ControlAction::Right => controls.right = Some(keycode),
                                ControlAction::Jump => controls.jump = Some(keycode),
                            }
                            self.controls_waiting_input = None;
                            self.audio.play_menu_select();
                            _captured = true;
                            break;
                        }
                    }
                    if is_key_pressed(KeyCode::Escape) {
                        self.controls_waiting_input = None;
                        self.audio.play_menu_select();
                    }
                } else {
                    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                        if self.controls_selection > 0 {
                            self.controls_selection -= 1;
                            self.audio.play_menu_select();
                        }
                    }
                    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                        if self.controls_selection < 2 {
                            self.controls_selection += 1;
                            self.audio.play_menu_select();
                        }
                    }
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                        if self.controls_player == 2 {
                            self.controls_player = 1;
                            self.controls_selection = 0;
                            self.audio.play_menu_select();
                        }
                    }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                        if self.controls_player == 1 {
                            self.controls_player = 2;
                            self.controls_selection = 0;
                            self.audio.play_menu_select();
                        }
                    }
                    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                        let action = match self.controls_selection {
                            0 => ControlAction::Left,
                            1 => ControlAction::Right,
                            2 => ControlAction::Jump,
                            _ => return,
                        };
                        self.controls_waiting_input = Some((self.controls_player, action));
                        self.audio.play_menu_select();
                    }
                    if is_key_pressed(KeyCode::Escape) {
                        self.state = GameState::Settings;
                        self.controls_waiting_input = None;
                        self.audio.play_menu_select();
                    }
                }
            }
            GameState::Credits => {
                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                {
                    if self.came_from_pause {
                        self.came_from_pause = false;
                        self.state = GameState::Pause;
                    } else {
                        self.state = GameState::Menu;
                        self.menu_selection = 0;
                    }
                }
            }
            GameState::Tutorial => {
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
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    if self.tutorial_page == TUTORIAL_PAGE_COUNT - 1 {
                        self.tutorial_completed = true;
                        self.start_transition(GameState::LevelSelect);
                    } else {
                        if self.tutorial_page < TUTORIAL_PAGE_COUNT - 1 {
                            self.tutorial_page += 1;
                        }
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.start_transition(GameState::LevelSelect);
                }
            }
            GameState::LevelSelect => {
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    for i in (0..self.level_selection).rev() {
                        if self.unlocked_levels[i] {
                            self.level_selection = i;
                            break;
                        }
                    }
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    for i in (self.level_selection + 1)..self.unlocked_levels.len() {
                        if self.unlocked_levels[i] {
                            self.level_selection = i;
                            break;
                        }
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    if self.level_selection < self.unlocked_levels.len()
                        && self.unlocked_levels[self.level_selection]
                    {
                        if self.level_selection == 0 && !self.tutorial_completed {
                            self.start_transition(GameState::Tutorial);
                        } else {
                            self.last_checkpoint_pos = None;
                            self.load_level(self.level_selection + 1, false, None, None);
                            self.score = 0;
                            if matches!(self.state, GameState::LevelSelect) {
                                let coming_from_coop = self.menu_selection == 2;
                                if coming_from_coop {
                                    self.player2 = Some(Player::new(
                                        150.0,
                                        GROUND_Y - PLAYER_HEIGHT,
                                        self.player_sprite_texture_p1
                                            .as_ref()
                                            .map(|t| std::rc::Rc::clone(t)),
                                        self.player_sprite_texture_p2
                                            .as_ref()
                                            .map(|t| std::rc::Rc::clone(t)),
                                    ));
                                    if let Some(ref mut p2) = self.player2 {
                                        p2.on_ground = true;
                                        p2.vel_y = 0.0;
                                    }
                                    self.start_transition(GameState::Coop);
                                } else {
                                    self.start_transition(GameState::Playing);
                                }
                            } else {
                                self.start_transition(GameState::Playing);
                            }
                        }
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Menu;
                    self.menu_selection = 0;
                }
            }
            GameState::Playing => {
                let effective_dt = if self.assist_mode {
                    dt * ASSIST_MODE_SLOW_MOTION
                } else {
                    dt
                };
                if is_key_pressed(KeyCode::P) {
                    self.state = GameState::Pause;
                    self.pause_selection = 0;
                    self.came_from_pause = false;
                    return;
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.transition_to_menu();
                    self.menu_selection = 0;
                    return;
                }
                if self.level_start_fade_timer > 0.0 {
                    self.level_start_fade_timer -= dt;
                }
                self.time_remaining -= effective_dt;
                if self.time_remaining <= 0.0 {
                    self.time_remaining = 0.0;
                    self.audio.play_death();
                    self.state = GameState::GameOver;
                    return;
                }
                let p1_left = self.is_control_pressed(&self.player1_controls, ControlAction::Left);
                let p1_right =
                    self.is_control_pressed(&self.player1_controls, ControlAction::Right);
                self.player.handle_movement_custom(p1_left, p1_right);
                self.player.update(effective_dt);
                let (px, py, pw, ph) = self.player.get_rect();
                for checkpoint in &mut self.checkpoints {
                    if checkpoint.check_activation(px, py, pw, ph) {
                        self.last_checkpoint_pos = Some((checkpoint.x, checkpoint.y));
                        self.score += SCORE_CHECKPOINT;
                        self.audio.play_coin();
                    }
                }
                Self::check_player_platform_collisions(
                    &mut self.player,
                    &self.platforms,
                    (px, py, pw, ph),
                );
                let p1_jump = self.is_control_pressed(&self.player1_controls, ControlAction::Jump);
                let jumped = self.player.handle_jump_custom(p1_jump);
                if jumped {
                    self.audio.play_jump(self.is_easter_egg());
                    self.camera_shake.trigger_jump();
                }
                self.player.update_animation(dt);
                if self.player.on_ground && self.player.vel_x.abs() > MIN_VELOCITY_FOR_FOOTSTEP {
                    self.footstep_timer += effective_dt;
                    if self.footstep_timer >= FOOTSTEP_INTERVAL {
                        self.audio.play_footstep(self.is_easter_egg());
                        self.footstep_timer = 0.0;
                        self.camera_shake.trigger_footstep();
                    }
                } else {
                    self.footstep_timer = 0.0;
                }
                let player_left = self.player.x;
                let player_right = self.player.x + self.player.width;
                if player_left < 0.0 {
                    self.player.x = 0.0;
                    self.player.vel_x = 0.0;
                }
                if player_right > WORLD_WIDTH {
                    self.player.x = WORLD_WIDTH - self.player.width;
                    self.player.vel_x = 0.0;
                }
                if self.player.y > FALL_DEATH_Y {
                    self.handle_player_death();
                }
                let mut player_vel_y_update = None;
                for enemy in &mut self.enemies {
                    if !enemy.alive {
                        continue;
                    }
                    enemy.update(effective_dt);
                    Self::check_enemy_platform_collisions(enemy, &self.platforms);
                    if enemy.on_ground {
                        enemy.check_edge(&self.platforms);
                    }
                    enemy.check_ground_collision(GROUND_Y);
                    match enemy.check_player_collision(px, py, pw, ph, self.player.vel_y) {
                        Some(true) => {
                            self.handle_player_death();
                            break;
                        }
                        Some(false) => {
                            let enemy_x = enemy.x;
                            let enemy_y = enemy.y;
                            let enemy_width = enemy.width;
                            let enemy_height = enemy.height;
                            player_vel_y_update = Some(JUMP_FORCE * JUMP_BOUNCE_MULTIPLIER);
                            self.audio.play_enemy_death();
                            self.score += SCORE_ENEMY;
                            self.camera_shake.trigger_kill();
                            for _ in 0..PARTICLE_COUNT {
                                let angle = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
                                let speed = rand::gen_range(40.0, 100.0);
                                self.particles.push(Particle::new(
                                    enemy_x + enemy_width / 2.0,
                                    enemy_y + enemy_height / 2.0,
                                    angle.cos() * speed,
                                    angle.sin() * speed,
                                ));
                            }
                        }
                        None => {}
                    }
                }
                if let Some(vel_y) = player_vel_y_update {
                    self.player.vel_y = vel_y;
                }
                let mut coins_to_collect = Vec::new();
                for coin in &mut self.coins {
                    if coin.collected {
                        continue;
                    }
                    coin.update(effective_dt);
                    if coin.check_collection(px, py, pw, ph) {
                        coins_to_collect.push((coin.x, coin.y));
                    }
                }
                for (coin_x, coin_y) in coins_to_collect {
                    self.handle_coin_collection(coin_x, coin_y);
                }
                if self.player.x > LEVEL_COMPLETE_X || self.coins_collected >= self.total_coins {
                    let time_bonus = (self.time_remaining * SCORE_TIME_BONUS) as u32;
                    self.score += SCORE_LEVEL_COMPLETE + time_bonus;
                    self.audio.play_level_complete();
                    if self.current_level < MAX_LEVELS
                        && self.current_level < self.unlocked_levels.len()
                    {
                        self.unlocked_levels[self.current_level] = true;
                    }
                    if let Err(e) = self.save_game(0) {
                        let error_msg = format!("Error saving game: {}", e);
                        eprintln!("{}", error_msg);
                        self.show_error(error_msg);
                    }
                    self.state = GameState::LevelComplete;
                }
                let screen_width = screen_width();
                self.camera
                    .update(self.player.x, screen_width, self.camera_shake.get_offset());
            }
            GameState::LevelComplete => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.state = GameState::LevelSelect;
                    if self.current_level < MAX_LEVELS
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
                if is_key_pressed(KeyCode::Escape) {
                    self.transition_to_menu();
                    self.menu_selection = 0;
                    self.player2 = None;
                    return;
                }
                let effective_dt = if self.assist_mode {
                    dt * ASSIST_MODE_SLOW_MOTION
                } else {
                    dt
                };
                self.versus_time_remaining -= effective_dt;
                if self.versus_time_remaining <= 0.0 {
                    self.versus_time_remaining = 0.0;
                    self.audio.play_level_complete();
                    self.state = GameState::VersusEnd;
                    return;
                }
                if self.respawn_timer_p1 > 0.0 {
                    self.respawn_timer_p1 -= dt;
                    if self.respawn_timer_p1 <= 0.0 {
                        self.player = Player::new(
                            100.0,
                            GROUND_Y - PLAYER_HEIGHT,
                            self.player_sprite_texture_p1
                                .as_ref()
                                .map(|t| std::rc::Rc::clone(t)),
                            self.player_sprite_texture_p2
                                .as_ref()
                                .map(|t| std::rc::Rc::clone(t)),
                        );
                        self.player.on_ground = true;
                        self.player.vel_y = 0.0;
                        self.player.y = GROUND_Y - PLAYER_HEIGHT;
                    }
                }
                if self.respawn_timer_p2 > 0.0 {
                    self.respawn_timer_p2 -= dt;
                    if let Some(ref mut p2) = self.player2 {
                        if self.respawn_timer_p2 <= 0.0 {
                            *p2 = Player::new(
                                700.0,
                                GROUND_Y - PLAYER_HEIGHT,
                                self.player_sprite_texture_p1
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                                self.player_sprite_texture_p2
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                            );
                            p2.on_ground = true;
                            p2.vel_y = 0.0;
                            p2.y = GROUND_Y - PLAYER_HEIGHT;
                        }
                    }
                }
                if self.respawn_timer_p1 <= 0.0 {
                    let p1_left =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Left);
                    let p1_right =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Right);
                    let p1_jump =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Jump);
                    let use_easter_egg = self.is_easter_egg();
                    let jumped = Self::update_versus_player_physics(
                        &mut self.player,
                        p1_left,
                        p1_right,
                        p1_jump,
                        &self.versus_platforms,
                        effective_dt,
                    );
                    if jumped {
                        self.audio.play_jump(use_easter_egg);
                    }
                    if self.player.on_ground && self.player.vel_x.abs() > MIN_VELOCITY_FOR_FOOTSTEP
                    {
                        self.footstep_timer += effective_dt;
                        if self.footstep_timer >= FOOTSTEP_INTERVAL {
                            self.audio.play_footstep(use_easter_egg);
                            self.footstep_timer = 0.0;
                        }
                    } else {
                        self.footstep_timer = 0.0;
                    }
                }
                let p2_controls = self.player2_controls.clone();
                let p2_left = self.is_control_pressed(&p2_controls, ControlAction::Left);
                let p2_right = self.is_control_pressed(&p2_controls, ControlAction::Right);
                let p2_jump = self.is_control_pressed(&p2_controls, ControlAction::Jump);
                if let Some(ref mut p2) = self.player2 {
                    if self.respawn_timer_p2 <= 0.0 {
                        let jumped = Self::update_versus_player_physics(
                            p2,
                            p2_left,
                            p2_right,
                            p2_jump,
                            &self.versus_platforms,
                            effective_dt,
                        );
                        if jumped {
                            self.audio.play_jump(false);
                        }
                    } else {
                        p2.update(effective_dt);
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        Self::check_player_platform_collisions(
                            p2,
                            &self.versus_platforms,
                            (px2, py2, pw2, ph2),
                        );
                    }
                }
                if self.respawn_timer_p1 > 0.0 {
                    self.player.update(effective_dt);
                    let (px, py, pw, ph) = self.player.get_rect();
                    Self::check_player_platform_collisions(
                        &mut self.player,
                        &self.versus_platforms,
                        (px, py, pw, ph),
                    );
                }
                if self.respawn_timer_p1 <= 0.0 && self.respawn_timer_p2 <= 0.0 {
                    if let Some(ref mut p2) = self.player2 {
                        if self.player.check_stomp(p2, self.player.vel_y) {
                            self.player1_score += 1;
                            self.player1_streak += 1;
                            self.player2_streak = 0;
                            let points = Self::calculate_versus_points(self.player1_streak);
                            self.player1_points += points;
                            self.audio.play_enemy_death();
                            self.respawn_timer_p2 = 2.0;
                            self.player.vel_y = JUMP_FORCE * JUMP_BOUNCE_MULTIPLIER;
                        } else if p2.check_stomp(&self.player, p2.vel_y) {
                            self.player2_score += 1;
                            self.player2_streak += 1;
                            self.player1_streak = 0;
                            let points = Self::calculate_versus_points(self.player2_streak);
                            self.player2_points += points;
                            self.audio.play_enemy_death();
                            self.respawn_timer_p1 = 2.0;
                            p2.vel_y = JUMP_FORCE * 0.6;
                        }
                    }
                }
                let p1_left = self.player.x;
                let p1_right = self.player.x + self.player.width;
                if p1_left < 0.0 {
                    self.player.x = 0.0;
                    self.player.vel_x = 0.0;
                }
                if p1_right > WORLD_WIDTH {
                    self.player.x = WORLD_WIDTH - self.player.width;
                    self.player.vel_x = 0.0;
                }
                if let Some(ref mut p2) = self.player2 {
                    let p2_left = p2.x;
                    let p2_right = p2.x + p2.width;
                    if p2_left < 0.0 {
                        p2.x = 0.0;
                        p2.vel_x = 0.0;
                    }
                    if p2_right > WORLD_WIDTH {
                        p2.x = WORLD_WIDTH - p2.width;
                        p2.vel_x = 0.0;
                    }
                }
                if self.player.y > FALL_DEATH_Y && self.respawn_timer_p1 <= 0.0 {
                    self.player2_score += 1;
                    self.player2_streak += 1;
                    self.player1_streak = 0;
                    let points = Self::calculate_versus_points(self.player2_streak);
                    self.player2_points += points;
                    self.audio.play_enemy_death();
                    self.respawn_timer_p1 = 2.0;
                }
                if let Some(ref p2) = self.player2 {
                    if p2.y > FALL_DEATH_Y && self.respawn_timer_p2 <= 0.0 {
                        self.player1_score += 1;
                        self.player1_streak += 1;
                        self.player2_streak = 0;
                        let points = Self::calculate_versus_points(self.player1_streak);
                        self.player1_points += points;
                        self.audio.play_enemy_death();
                        self.respawn_timer_p2 = 2.0;
                    }
                }
                if let Some(ref p2) = self.player2 {
                    let center_x = (self.player.x + p2.x) / 2.0;
                    let screen_width = screen_width();
                    let shake = self.camera_shake.get_offset();
                    self.camera.update(center_x, screen_width, shake);
                }
            }
            GameState::VersusEnd => {
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Escape)
                {
                    self.state = GameState::Menu;
                    self.menu_selection = 0;
                    self.player2 = None;
                }
            }
            GameState::Coop => {
                let effective_dt = if self.assist_mode {
                    dt * ASSIST_MODE_SLOW_MOTION
                } else {
                    dt
                };
                if is_key_pressed(KeyCode::P) {
                    self.state = GameState::Pause;
                    self.pause_selection = 0;
                    self.came_from_pause = false;
                    return;
                }
                if is_key_pressed(KeyCode::Escape) {
                    self.transition_to_menu();
                    self.menu_selection = 0;
                    self.player2 = None;
                    return;
                }
                if self.level_start_fade_timer > 0.0 {
                    self.level_start_fade_timer -= dt;
                }
                self.time_remaining -= effective_dt;
                if self.time_remaining <= 0.0 {
                    self.time_remaining = 0.0;
                    self.audio.play_death();
                    self.state = GameState::GameOver;
                    return;
                }
                if self.respawn_timer_p1 > 0.0 {
                    self.respawn_timer_p1 -= dt;
                    if self.respawn_timer_p1 <= 0.0 {
                        if let Some(checkpoint) = self.last_checkpoint_pos {
                            self.player = Player::new(
                                checkpoint.0,
                                checkpoint.1 - PLAYER_HEIGHT,
                                self.player_sprite_texture_p1
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                                self.player_sprite_texture_p2
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                            );
                        } else {
                            self.player = Player::new(
                                50.0,
                                GROUND_Y - PLAYER_HEIGHT,
                                self.player_sprite_texture_p1
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                                self.player_sprite_texture_p2
                                    .as_ref()
                                    .map(|t| std::rc::Rc::clone(t)),
                            );
                        }
                        self.player.on_ground = true;
                        self.player.vel_y = 0.0;
                    }
                }
                if self.respawn_timer_p2 > 0.0 {
                    self.respawn_timer_p2 -= dt;
                    if let Some(ref mut p2) = self.player2 {
                        if self.respawn_timer_p2 <= 0.0 {
                            if let Some(checkpoint) = self.last_checkpoint_pos {
                                *p2 = Player::new(
                                    checkpoint.0 + 100.0,
                                    checkpoint.1 - PLAYER_HEIGHT,
                                    self.player_sprite_texture_p1
                                        .as_ref()
                                        .map(|t| std::rc::Rc::clone(t)),
                                    self.player_sprite_texture_p2
                                        .as_ref()
                                        .map(|t| std::rc::Rc::clone(t)),
                                );
                            } else {
                                *p2 = Player::new(
                                    150.0,
                                    GROUND_Y - PLAYER_HEIGHT,
                                    self.player_sprite_texture_p1
                                        .as_ref()
                                        .map(|t| std::rc::Rc::clone(t)),
                                    self.player_sprite_texture_p2
                                        .as_ref()
                                        .map(|t| std::rc::Rc::clone(t)),
                                );
                            }
                            p2.on_ground = true;
                            p2.vel_y = 0.0;
                        }
                    }
                }
                if self.respawn_timer_p1 <= 0.0 {
                    let p1_left =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Left);
                    let p1_right =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Right);
                    self.player.handle_movement_custom(p1_left, p1_right);
                    self.player.update(effective_dt);
                    let (px, py, pw, ph) = self.player.get_rect();
                    for checkpoint in &mut self.checkpoints {
                        if checkpoint.check_activation(px, py, pw, ph) {
                            self.last_checkpoint_pos = Some((checkpoint.x, checkpoint.y));
                            self.score += SCORE_CHECKPOINT;
                            self.audio.play_coin();
                        }
                    }
                    Self::check_player_platform_collisions(
                        &mut self.player,
                        &self.platforms,
                        (px, py, pw, ph),
                    );
                    let p1_jump =
                        self.is_control_pressed(&self.player1_controls, ControlAction::Jump);
                    let jumped = self.player.handle_jump_custom(p1_jump);
                    if jumped {
                        self.audio.play_jump(self.is_easter_egg());
                    }
                    self.player.update_animation(dt);
                    if self.player.on_ground && self.player.vel_x.abs() > MIN_VELOCITY_FOR_FOOTSTEP
                    {
                        self.footstep_timer += effective_dt;
                        if self.footstep_timer >= FOOTSTEP_INTERVAL {
                            self.audio.play_footstep(self.is_easter_egg());
                            self.footstep_timer = 0.0;
                        }
                    } else {
                        self.footstep_timer = 0.0;
                    }
                }
                let p2_controls = self.player2_controls.clone();
                let p2_left = self.is_control_pressed(&p2_controls, ControlAction::Left);
                let p2_right = self.is_control_pressed(&p2_controls, ControlAction::Right);
                let p2_jump = self.is_control_pressed(&p2_controls, ControlAction::Jump);
                if let Some(ref mut p2) = self.player2 {
                    if self.respawn_timer_p2 <= 0.0 {
                        p2.handle_movement_custom(p2_left, p2_right);
                        p2.update(effective_dt);
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        for checkpoint in &mut self.checkpoints {
                            if checkpoint.check_activation(px2, py2, pw2, ph2) {
                                self.last_checkpoint_pos = Some((checkpoint.x, checkpoint.y));
                                self.score += SCORE_CHECKPOINT;
                                self.audio.play_coin();
                            }
                        }
                        Self::check_player_platform_collisions(
                            p2,
                            &self.platforms,
                            (px2, py2, pw2, ph2),
                        );
                        let jumped = p2.handle_jump_custom(p2_jump);
                        if jumped {
                            self.audio.play_jump(false);
                        }
                        p2.update_animation(effective_dt);
                    } else {
                        p2.update(effective_dt);
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        Self::check_player_platform_collisions(
                            p2,
                            &self.platforms,
                            (px2, py2, pw2, ph2),
                        );
                    }
                }
                if self.respawn_timer_p1 > 0.0 {
                    self.player.update(effective_dt);
                    let (px, py, pw, ph) = self.player.get_rect();
                    Self::check_player_platform_collisions(
                        &mut self.player,
                        &self.platforms,
                        (px, py, pw, ph),
                    );
                }
                let player_left = self.player.x;
                let player_right = self.player.x + self.player.width;
                if player_left < 0.0 {
                    self.player.x = 0.0;
                    self.player.vel_x = 0.0;
                }
                if player_right > WORLD_WIDTH {
                    self.player.x = WORLD_WIDTH - self.player.width;
                    self.player.vel_x = 0.0;
                }
                if let Some(ref mut p2) = self.player2 {
                    let p2_left = p2.x;
                    let p2_right = p2.x + p2.width;
                    if p2_left < 0.0 {
                        p2.x = 0.0;
                        p2.vel_x = 0.0;
                    }
                    if p2_right > WORLD_WIDTH {
                        p2.x = WORLD_WIDTH - p2.width;
                        p2.vel_x = 0.0;
                    }
                }
                if self.player.y > FALL_DEATH_Y && self.respawn_timer_p1 <= 0.0 {
                    self.audio.play_death();
                    if self.lives > 0 {
                        self.lives -= 1;
                    }
                    if self.lives == 0 {
                        self.game_over_fade_timer = GAME_OVER_FADE_TIMER;
                        self.state = GameState::GameOver;
                    } else {
                        self.respawn_timer_p1 = RESPAWN_TIMER;
                    }
                }
                if let Some(ref p2) = self.player2 {
                    if p2.y > FALL_DEATH_Y && self.respawn_timer_p2 <= 0.0 {
                        self.audio.play_death();
                        if self.lives > 0 {
                            self.lives -= 1;
                        }
                        if self.lives == 0 {
                            self.game_over_fade_timer = GAME_OVER_FADE_TIMER;
                            self.state = GameState::GameOver;
                        } else {
                            self.respawn_timer_p2 = RESPAWN_TIMER;
                        }
                    }
                }
                let mut player_vel_y_update = None;
                let mut player2_vel_y_update = None;
                for enemy in &mut self.enemies {
                    if !enemy.alive {
                        continue;
                    }
                    enemy.update(effective_dt);
                    Self::check_enemy_platform_collisions(enemy, &self.platforms);
                    if enemy.on_ground {
                        enemy.check_edge(&self.platforms);
                    }
                    enemy.check_ground_collision(GROUND_Y);
                    let (px, py, pw, ph) = self.player.get_rect();
                    match enemy.check_player_collision(px, py, pw, ph, self.player.vel_y) {
                        Some(true) => {
                            self.handle_player_death_versus_p1();
                            break;
                        }
                        Some(false) => {
                            let enemy_x = enemy.x;
                            let enemy_y = enemy.y;
                            let enemy_width = enemy.width;
                            let enemy_height = enemy.height;
                            player_vel_y_update = Some(JUMP_FORCE * JUMP_BOUNCE_MULTIPLIER);
                            self.audio.play_enemy_death();
                            self.score += SCORE_ENEMY;
                            self.camera_shake.trigger_kill();
                            for _ in 0..PARTICLE_COUNT {
                                let angle = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
                                let speed = rand::gen_range(40.0, 100.0);
                                self.particles.push(Particle::new(
                                    enemy_x + enemy_width / 2.0,
                                    enemy_y + enemy_height / 2.0,
                                    angle.cos() * speed,
                                    angle.sin() * speed,
                                ));
                            }
                        }
                        None => {}
                    }
                    if let Some(ref p2) = self.player2 {
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        match enemy.check_player_collision(px2, py2, pw2, ph2, p2.vel_y) {
                            Some(true) => {
                                self.handle_player_death_versus_p2();
                                break;
                            }
                            Some(false) => {
                                let enemy_x = enemy.x;
                                let enemy_y = enemy.y;
                                let enemy_width = enemy.width;
                                let enemy_height = enemy.height;
                                player2_vel_y_update = Some(JUMP_FORCE * JUMP_BOUNCE_MULTIPLIER);
                                self.audio.play_enemy_death();
                                self.score += SCORE_ENEMY;
                                self.camera_shake.trigger_kill();
                                for _ in 0..PARTICLE_COUNT {
                                    let angle = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
                                    let speed = rand::gen_range(40.0, 100.0);
                                    self.particles.push(Particle::new(
                                        enemy_x + enemy_width / 2.0,
                                        enemy_y + enemy_height / 2.0,
                                        angle.cos() * speed,
                                        angle.sin() * speed,
                                    ));
                                }
                            }
                            None => {}
                        }
                    }
                }
                if let Some(vel_y) = player_vel_y_update {
                    self.player.vel_y = vel_y;
                }
                if let Some(vel_y) = player2_vel_y_update {
                    if let Some(ref mut p2) = self.player2 {
                        p2.vel_y = vel_y;
                    }
                }
                let mut coins_to_collect = Vec::new();
                for coin in &mut self.coins {
                    if coin.collected {
                        continue;
                    }
                    coin.update(effective_dt);
                    let (px, py, pw, ph) = self.player.get_rect();
                    if coin.check_collection(px, py, pw, ph) {
                        coins_to_collect.push((coin.x, coin.y));
                    }
                    if let Some(ref p2) = self.player2 {
                        let (px2, py2, pw2, ph2) = p2.get_rect();
                        if coin.check_collection(px2, py2, pw2, ph2) {
                            coins_to_collect.push((coin.x, coin.y));
                        }
                    }
                }
                for (coin_x, coin_y) in coins_to_collect {
                    self.handle_coin_collection(coin_x, coin_y);
                }
                if (self.player.x > LEVEL_COMPLETE_X || self.coins_collected >= self.total_coins)
                    && (self.player2.is_none()
                        || self
                            .player2
                            .as_ref()
                            .map(|p| {
                                p.x > LEVEL_COMPLETE_X || self.coins_collected >= self.total_coins
                            })
                            .unwrap_or(true))
                {
                    let time_bonus = (self.time_remaining * SCORE_TIME_BONUS) as u32;
                    self.score += SCORE_LEVEL_COMPLETE + time_bonus;
                    self.audio.play_level_complete();
                    if self.current_level < MAX_LEVELS
                        && self.current_level < self.unlocked_levels.len()
                    {
                        self.unlocked_levels[self.current_level] = true;
                    }
                    if let Err(e) = self.save_game(0) {
                        let error_msg = format!("Error saving game: {}", e);
                        eprintln!("{}", error_msg);
                        self.show_error(error_msg);
                    }
                    self.state = GameState::LevelComplete;
                }
                if let Some(ref p2) = self.player2 {
                    let center_x = (self.player.x + p2.x) / 2.0;
                    let screen_w = screen_width();
                    let shake = self.camera_shake.get_offset();
                    self.camera.update(center_x, screen_w, shake);
                } else {
                    let player_x = self.player.x;
                    let screen_w = screen_width();
                    let shake = self.camera_shake.get_offset();
                    self.camera.update(player_x, screen_w, shake);
                }
            }
            GameState::Pause => {
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if self.pause_selection > 0 {
                        self.pause_selection -= 1;
                        self.pause_animation.trigger(self.pause_selection);
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if self.pause_selection < 3 {
                        self.pause_selection += 1;
                        self.pause_animation.trigger(self.pause_selection);
                        self.audio.play_menu_select();
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.audio.play_menu_select();
                    match self.pause_selection {
                        0 => {
                            self.state = GameState::Playing;
                        }
                        1 => {
                            self.came_from_pause = true;
                            self.state = GameState::Settings;
                            self.settings_selection = 0;
                        }
                        2 => {
                            self.came_from_pause = true;
                            self.state = GameState::Credits;
                        }
                        3 => {
                            self.state = GameState::Menu;
                            self.menu_selection = 0;
                        }
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Playing;
                }
            }
            GameState::Respawn => {
                self.respawn_timer -= dt;
                if self.respawn_timer <= 0.0 {
                    self.respawn_timer = 0.0;
                    self.load_level(
                        self.current_level,
                        self.last_checkpoint_pos.is_some(),
                        Some(self.time_remaining),
                        Some(self.coins_collected),
                    );
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                if self.game_over_fade_timer > 0.0 {
                    self.game_over_fade_timer -= dt;
                }
                if self.game_over_fade_timer <= 0.0 {
                    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                        self.lives = DEFAULT_LIVES;
                        self.load_level(
                            self.current_level,
                            self.last_checkpoint_pos.is_some(),
                            None,
                            None,
                        );
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
}
