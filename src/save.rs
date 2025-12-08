use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveData {
    pub current_level: usize,
    pub unlocked_levels: Vec<bool>,
    pub lives: u32,
    pub score: u32,
    #[serde(default)]
    pub coins_collected: u32,
    #[serde(default)]
    pub total_coins: u32,
    #[serde(default)]
    pub time_remaining: f32,
    #[serde(default)]
    pub time_taken: f32,
    #[serde(default)]
    pub timestamp: u64,
    pub last_checkpoint_pos: Option<(f32, f32)>,
    #[serde(default)]
    pub player_name: String,
    #[serde(default)]
    pub tutorial_completed: bool,
    #[serde(default)]
    pub versus_played: bool,
}

impl SaveData {
    pub fn new() -> Self {
        let mut unlocked_levels = vec![false; crate::constants::MAX_LEVELS];
        unlocked_levels[0] = true;
        
        Self {
            current_level: 1,
            unlocked_levels,
            lives: 5,
            score: 0,
            coins_collected: 0,
            total_coins: 0,
            time_remaining: crate::constants::TIME_LIMIT,
            time_taken: 0.0,
            timestamp: 0,
            last_checkpoint_pos: None,
            player_name: String::new(),
            tutorial_completed: false,
            versus_played: false,
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Erro ao serializar save: {}", e))?;
        
        std::fs::write(path, json)
            .map_err(|e| format!("Erro ao escrever arquivo: {}", e))?;
        
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Erro ao ler arquivo: {}", e))?;
        
        let save_data: SaveData = serde_json::from_str(&content)
            .map_err(|e| format!("Erro ao deserializar save: {}", e))?;
        
        Ok(save_data)
    }

    pub fn save_exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    pub fn get_save_path(slot: usize) -> String {
        format!("save_slot{}.json", slot + 1)
    }

    pub fn list_all_saves() -> Vec<(usize, Option<SaveData>)> {
        let mut saves = Vec::new();
        for slot in 0..3 {
            let path = Self::get_save_path(slot);
            if Self::save_exists(&path) {
                match Self::load_from_file(&path) {
                    Ok(save_data) => saves.push((slot, Some(save_data))),
                    Err(_) => saves.push((slot, None)),
                }
            } else {
                saves.push((slot, None));
            }
        }
        saves
    }

    pub fn delete_save(slot: usize) -> Result<(), String> {
        let path = Self::get_save_path(slot);
        if Self::save_exists(&path) {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Erro ao apagar arquivo: {}", e))?;
        }
        Ok(())
    }
}
