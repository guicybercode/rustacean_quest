use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use directories::ProjectDirs;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SAVE_VERSION: u8 = 1;
const APP_QUALIFIER: &str = "";
const APP_ORG: &str = "JumpQuest";
const APP_NAME: &str = "JumpQuest";
const SAVE_PREFIX: &str = "profile";
const SAVE_EXTENSION: &str = "dat";
const KEY_ENV: &str = "JUMPQUEST_SAVE_KEY";
const SAVE_DIR_ENV: &str = "JUMPQUEST_SAVE_DIR";
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const MAX_LIVES: u32 = 99;
const LEGACY_PREFIX: &str = "save_slot";
const LEGACY_EXTENSION: &str = "json";

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

#[derive(Serialize, Deserialize)]
struct SaveBlob {
    version: u8,
    nonce: [u8; NONCE_LEN],
    ciphertext: Vec<u8>,
}

impl SaveData {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut unlocked_levels = vec![false; crate::constants::MAX_LEVELS];
        unlocked_levels[0] = true;

        Self {
            current_level: 1,
            unlocked_levels,
            lives: crate::constants::DEFAULT_LIVES,
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

    fn validate(mut self) -> Self {
        use crate::constants::{MAX_LEVELS, TIME_LIMIT};
        if self.current_level == 0 {
            self.current_level = 1;
        }
        if self.current_level > MAX_LEVELS {
            self.current_level = MAX_LEVELS;
        }
        self.lives = self.lives.min(MAX_LIVES);
        self.coins_collected = self.coins_collected.min(self.total_coins);
        if self.time_remaining.is_sign_negative() || !self.time_remaining.is_finite() {
            self.time_remaining = TIME_LIMIT;
        }
        if self.time_remaining > TIME_LIMIT {
            self.time_remaining = TIME_LIMIT;
        }
        if self.time_taken.is_sign_negative() || !self.time_taken.is_finite() {
            self.time_taken = 0.0;
        }
        self
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let key = load_key()?;
        let cipher = ChaCha20Poly1305::new(&key);
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let payload =
            bincode::serialize(self).map_err(|e| format!("Error serializing save: {e}"))?;
        let ciphertext = cipher
            .encrypt(nonce, payload.as_ref())
            .map_err(|e| format!("Error encrypting save: {e}"))?;

        let blob = SaveBlob {
            version: SAVE_VERSION,
            nonce: nonce_bytes,
            ciphertext,
        };
        let blob_bytes =
            bincode::serialize(&blob).map_err(|e| format!("Error encoding save blob: {e}"))?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Error creating save dir: {e}"))?;
        }
        fs::write(path, blob_bytes).map_err(|e| format!("Error writing save file: {e}"))?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| format!("Error reading save: {e}"))?;
        let blob: SaveBlob =
            bincode::deserialize(&bytes).map_err(|e| format!("Error decoding save blob: {e}"))?;
        if blob.version != SAVE_VERSION {
            return Err("Save version not supported".to_string());
        }

        let key = load_key()?;
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = Nonce::from_slice(&blob.nonce);
        let plaintext = cipher
            .decrypt(nonce, blob.ciphertext.as_ref())
            .map_err(|e| format!("Error decrypting save: {e}"))?;
        let save_data: SaveData = bincode::deserialize(&plaintext)
            .map_err(|e| format!("Error deserializing save: {e}"))?;
        Ok(save_data.validate())
    }

    pub fn load_slot(slot: usize) -> Result<Self, String> {
        if let Some(migrated) = ensure_migrated(slot)? {
            return Ok(migrated);
        }
        let path = Self::get_save_path(slot)?;
        Self::load_from_file(&path)
    }

    pub fn save_exists(path: &Path) -> bool {
        path.exists()
    }

    pub fn get_save_path(slot: usize) -> Result<PathBuf, String> {
        let base = get_save_dir()?;
        let filename = format!("{SAVE_PREFIX}_{slot}.{SAVE_EXTENSION}");
        Ok(base.join(filename))
    }

    pub fn list_all_saves() -> Vec<(usize, Option<SaveData>)> {
        let mut saves = Vec::with_capacity(crate::constants::MAX_SAVE_SLOTS);
        for slot in 0..crate::constants::MAX_SAVE_SLOTS {
            match ensure_migrated(slot) {
                Ok(Some(data)) => {
                    saves.push((slot, Some(data)));
                    continue;
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Error migrating legacy save slot {}: {}", slot, e);
                }
            }
            match Self::get_save_path(slot) {
                Ok(path) => {
                    if Self::save_exists(&path) {
                        match Self::load_from_file(&path) {
                            Ok(save_data) => saves.push((slot, Some(save_data))),
                            Err(e) => {
                                eprintln!("Error reading save slot {}: {}", slot, e);
                                saves.push((slot, None));
                            }
                        }
                    } else {
                        saves.push((slot, None));
                    }
                }
                Err(e) => {
                    eprintln!("Error resolving save slot {}: {}", slot, e);
                    saves.push((slot, None));
                }
            }
        }
        saves
    }

    pub fn delete_save(slot: usize) -> Result<(), String> {
        let path = Self::get_save_path(slot)?;
        if Self::save_exists(&path) {
            fs::remove_file(&path).map_err(|e| format!("Error deleting file: {e}"))?;
        }
        let legacy = legacy_save_path(slot);
        if legacy.exists() {
            let _ = fs::remove_file(&legacy);
        }
        Ok(())
    }

    pub fn legacy_exists(slot: usize) -> bool {
        legacy_save_path(slot).exists()
    }
}

fn load_key() -> Result<Key, String> {
    let encoded = env::var(KEY_ENV).map_err(|_| {
        format!("Missing env {KEY_ENV} with 32-byte base64 key (base64 of 32 random bytes)")
    })?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("Invalid base64 in {KEY_ENV}: {e}"))?;
    if bytes.len() != KEY_LEN {
        return Err(format!(
            "{KEY_ENV} must be {} bytes after base64 decoding",
            KEY_LEN
        ));
    }
    let mut key_bytes = [0u8; KEY_LEN];
    key_bytes.copy_from_slice(&bytes);
    Ok(Key::from_slice(&key_bytes).to_owned())
}

fn get_save_dir() -> Result<PathBuf, String> {
    if let Ok(dir) = env::var(SAVE_DIR_ENV) {
        return Ok(PathBuf::from(dir));
    }
    let dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
        .ok_or_else(|| "No valid save dir".to_string())?;
    Ok(dirs.data_dir().to_path_buf())
}

fn legacy_save_path(slot: usize) -> PathBuf {
    let filename = format!("{LEGACY_PREFIX}{}.{}", slot + 1, LEGACY_EXTENSION);
    PathBuf::from(filename)
}

fn ensure_migrated(slot: usize) -> Result<Option<SaveData>, String> {
    let new_path = SaveData::get_save_path(slot)?;
    if SaveData::save_exists(&new_path) {
        return Ok(None);
    }
    let legacy_path = legacy_save_path(slot);
    if !legacy_path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&legacy_path)
        .map_err(|e| format!("Error reading legacy save {:?}: {}", legacy_path, e))?;
    let legacy: SaveData = serde_json::from_str(&content)
        .map_err(|e| format!("Error parsing legacy save {:?}: {}", legacy_path, e))?;
    let validated = legacy.clone().validate();
    validated.save_to_file(&new_path)?;
    let _ = fs::remove_file(&legacy_path);
    Ok(Some(validated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::tempdir;

    fn set_test_key() -> String {
        // 32 bytes of zero for deterministic tests
        base64::engine::general_purpose::STANDARD.encode([0u8; KEY_LEN])
    }

    #[test]
    fn validate_caps_values() {
        let data = SaveData {
            current_level: 999,
            unlocked_levels: vec![true; crate::constants::MAX_LEVELS + 2],
            lives: 500,
            score: 0,
            coins_collected: 100,
            total_coins: 10,
            time_remaining: f32::INFINITY,
            time_taken: f32::NAN,
            timestamp: 0,
            last_checkpoint_pos: None,
            player_name: String::new(),
            tutorial_completed: false,
            versus_played: false,
        };
        let v = data.validate();
        assert!(v.current_level <= crate::constants::MAX_LEVELS);
        assert!(v.lives <= MAX_LIVES);
        assert!(v.coins_collected <= v.total_coins);
        assert!(v.time_remaining <= crate::constants::TIME_LIMIT);
        assert!(v.time_taken >= 0.0);
    }

    #[test]
    #[serial]
    fn save_round_trip_encrypted() {
        let dir = tempdir().unwrap();
        env::set_var(SAVE_DIR_ENV, dir.path());
        env::set_var(KEY_ENV, set_test_key());
        let data = SaveData::new();
        let path = SaveData::get_save_path(0).unwrap();
        data.save_to_file(&path).unwrap();
        let loaded = SaveData::load_from_file(&path).unwrap();
        assert_eq!(loaded.current_level, data.current_level);
        assert_eq!(loaded.unlocked_levels, data.unlocked_levels);
    }

    #[test]
    #[serial]
    fn save_fails_without_key() {
        let dir = tempdir().unwrap();
        env::set_var(SAVE_DIR_ENV, dir.path());
        env::remove_var(KEY_ENV);
        let data = SaveData::new();
        let path = SaveData::get_save_path(0).unwrap();
        let err = data.save_to_file(&path).unwrap_err();
        assert!(err.contains("Missing env"));
    }

    #[test]
    #[serial]
    fn save_fails_with_bad_key() {
        let dir = tempdir().unwrap();
        env::set_var(SAVE_DIR_ENV, dir.path());
        env::set_var(KEY_ENV, "dQ=="); // 1 byte
        let data = SaveData::new();
        let path = SaveData::get_save_path(0).unwrap();
        let err = data.save_to_file(&path).unwrap_err();
        assert!(err.contains("must be 32 bytes"));
    }

    #[test]
    #[serial]
    fn migrates_legacy_json() {
        let dir = tempdir().unwrap();
        env::set_var(SAVE_DIR_ENV, dir.path());
        env::set_var(KEY_ENV, set_test_key());
        let cwd = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let legacy_path = legacy_save_path(0);
        let legacy = SaveData::new();
        let json = serde_json::to_string(&legacy).unwrap();
        fs::write(&legacy_path, json).unwrap();

        let loaded = SaveData::load_slot(0).unwrap();
        let new_path = SaveData::get_save_path(0).unwrap();
        assert!(new_path.exists());
        assert!(!legacy_path.exists());
        assert_eq!(loaded.current_level, legacy.current_level);

        env::set_current_dir(cwd).unwrap();
    }
}
