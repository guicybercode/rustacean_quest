use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

pub struct AudioManager {
    jump_sound: Option<Sound>,
    coin_sound: Option<Sound>,
    death_sound: Option<Sound>,
    enemy_death_sound: Option<Sound>,
    level_complete_sound: Option<Sound>,
    menu_select_sound: Option<Sound>,
    footstep_sound: Option<Sound>,
    jump_sound_special: Option<Sound>,
    footstep_sound_special: Option<Sound>,
    sounds_enabled: bool,
    load_errors: Vec<String>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            jump_sound: None,
            coin_sound: None,
            death_sound: None,
            enemy_death_sound: None,
            level_complete_sound: None,
            menu_select_sound: None,
            footstep_sound: None,
            jump_sound_special: None,
            footstep_sound_special: None,
            sounds_enabled: true,
            load_errors: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.sounds_enabled
    }

    #[allow(dead_code)]
    pub fn toggle(&mut self) -> bool {
        self.sounds_enabled = !self.sounds_enabled;
        self.sounds_enabled
    }

    #[allow(dead_code)]
    pub fn get_load_errors(&self) -> &[String] {
        &self.load_errors
    }

    #[allow(dead_code)]
    pub fn all_sounds_loaded(&self) -> bool {
        self.jump_sound.is_some()
            && self.coin_sound.is_some()
            && self.death_sound.is_some()
            && self.enemy_death_sound.is_some()
            && self.level_complete_sound.is_some()
            && self.menu_select_sound.is_some()
            && self.footstep_sound.is_some()
    }

    async fn load_sound_safe(bytes: &[u8], name: &str) -> Result<Sound, String> {
        load_sound_from_bytes(bytes)
            .await
            .map_err(|e| format!("Error loading sound '{}': {:?}", name, e))
    }

    fn generate_beep_bytes(frequency: f32, duration: f32, sample_rate: u32) -> Vec<u8> {
        let num_samples = (sample_rate as f32 * duration) as usize;
        let mut samples = Vec::with_capacity(num_samples * 2);

        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let envelope = if num_samples > 20 {
                if i < num_samples / 10 {
                    i as f32 / (num_samples / 10) as f32
                } else if i > num_samples * 9 / 10 {
                    (num_samples - i) as f32 / (num_samples / 10) as f32
                } else {
                    1.0
                }
            } else {
                1.0
            };

            let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin() * envelope * 0.5;
            let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;

            samples.push((sample_i16 & 0xFF) as u8);
            samples.push((sample_i16 >> 8) as u8);
        }

        let mut wav = Vec::new();
        let data_size = samples.len() as u32;
        let file_size = 36 + data_size;

        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&file_size.to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&((sample_rate * 2) as u32).to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());

        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());
        wav.extend_from_slice(&samples);

        wav
    }

    pub async fn load_sounds(&mut self) {
        self.load_errors.clear();

        let jump_bytes = Self::generate_beep_bytes(440.0, 0.2, 44100);
        let coin_bytes = Self::generate_beep_bytes(880.0, 0.25, 44100);
        let death_bytes = Self::generate_beep_bytes(220.0, 0.6, 44100);
        let enemy_death_bytes = Self::generate_beep_bytes(330.0, 0.3, 44100);
        let level_complete_bytes = Self::generate_beep_bytes(523.25, 0.4, 44100);
        let menu_select_bytes = Self::generate_beep_bytes(600.0, 0.15, 44100);
        let footstep_bytes = Self::generate_beep_bytes(200.0, 0.08, 44100);

        let jump_special_bytes = Self::generate_beep_bytes(660.0, 0.15, 44100);
        let footstep_special_bytes = Self::generate_beep_bytes(350.0, 0.1, 44100);

        match Self::load_sound_safe(&jump_bytes, "jump").await {
            Ok(sound) => self.jump_sound = Some(sound),
            Err(e) => {
                eprintln!("{}", e);
                self.load_errors.push(e);
            }
        }

        match Self::load_sound_safe(&coin_bytes, "coin").await {
            Ok(sound) => self.coin_sound = Some(sound),
            Err(e) => {
                eprintln!("{}", e);
                self.load_errors.push(e);
            }
        }

        match Self::load_sound_safe(&death_bytes, "death").await {
            Ok(sound) => self.death_sound = Some(sound),
            Err(e) => {
                eprintln!("{}", e);
                self.load_errors.push(e);
            }
        }

        match Self::load_sound_safe(&enemy_death_bytes, "enemy_death").await {
            Ok(sound) => self.enemy_death_sound = Some(sound),
            Err(e) => {
                eprintln!("{}", e);
                self.load_errors.push(e);
            }
        }

        match Self::load_sound_safe(&level_complete_bytes, "level_complete").await {
            Ok(sound) => self.level_complete_sound = Some(sound),
            Err(e) => {
                eprintln!("{}", e);
                self.load_errors.push(e);
            }
        }

        match Self::load_sound_safe(&menu_select_bytes, "menu_select").await {
            Ok(sound) => self.menu_select_sound = Some(sound),
            Err(e) => {
                eprintln!("{}", e);
                self.load_errors.push(e);
            }
        }

        match Self::load_sound_safe(&footstep_bytes, "footstep").await {
            Ok(sound) => self.footstep_sound = Some(sound),
            Err(e) => {
                eprintln!("{}", e);
                self.load_errors.push(e);
            }
        }

        match Self::load_sound_safe(&jump_special_bytes, "jump_special").await {
            Ok(sound) => self.jump_sound_special = Some(sound),
            Err(e) => {
                eprintln!("{}", e);
                self.load_errors.push(e);
            }
        }

        match Self::load_sound_safe(&footstep_special_bytes, "footstep_special").await {
            Ok(sound) => self.footstep_sound_special = Some(sound),
            Err(e) => {
                eprintln!("{}", e);
                self.load_errors.push(e);
            }
        }
    }

    fn play_sound(sound: &Option<Sound>, volume: f32) {
        if let Some(s) = sound {
            play_sound(
                s,
                PlaySoundParams {
                    volume,
                    looped: false,
                },
            );
        }
    }

    pub fn play_jump(&self, special: bool) {
        if self.sounds_enabled {
            if special {
                Self::play_sound(&self.jump_sound_special, 0.7);
            } else {
                Self::play_sound(&self.jump_sound, 0.7);
            }
        }
    }

    pub fn play_coin(&self) {
        if self.sounds_enabled {
            Self::play_sound(&self.coin_sound, 0.9);
        }
    }

    pub fn play_death(&self) {
        if self.sounds_enabled {
            Self::play_sound(&self.death_sound, 0.85);
        }
    }

    pub fn play_enemy_death(&self) {
        if self.sounds_enabled {
            Self::play_sound(&self.enemy_death_sound, 0.9);
        }
    }

    pub fn play_level_complete(&self) {
        if self.sounds_enabled {
            Self::play_sound(&self.level_complete_sound, 0.85);
        }
    }

    pub fn play_menu_select(&self) {
        if self.sounds_enabled {
            Self::play_sound(&self.menu_select_sound, 0.6);
        }
    }

    pub fn play_footstep(&self, special: bool) {
        if self.sounds_enabled {
            if special {
                Self::play_sound(&self.footstep_sound_special, 0.15);
            } else {
                Self::play_sound(&self.footstep_sound, 0.15);
            }
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.sounds_enabled = enabled;
    }
}
