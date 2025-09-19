#![cfg(target_arch = "wasm32")]

use {
    web_sys::window,
    serde::{Serialize, Deserialize},
    crate::common::shared_types::{TuningSystem, Scale, MidiNote},
    crate::common::dev_log,
};

const STORAGE_KEY: &str = "intonation_toy_config";
const EXPIRATION_MS: i64 = 24 * 60 * 60 * 1000; // 24 hours in milliseconds

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredConfig {
    pub tonal_center_note: MidiNote,
    pub tuning_system: TuningSystem,
    pub scale: Scale,
    pub timestamp: i64,
}

impl StoredConfig {
    pub fn new(tonal_center_note: MidiNote, tuning_system: TuningSystem, scale: Scale) -> Self {
        let timestamp = js_sys::Date::now() as i64;
        Self {
            tonal_center_note,
            tuning_system,
            scale,
            timestamp,
        }
    }

    pub fn is_expired(&self) -> bool {
        let current_time = js_sys::Date::now() as i64;
        (current_time - self.timestamp) > EXPIRATION_MS
    }
}

pub fn save_config(tonal_center_note: MidiNote, tuning_system: TuningSystem, scale: Scale) {
    let Some(window) = window() else {
        dev_log!("Failed to get window for storage");
        return;
    };
    
    let Some(storage) = window.local_storage().ok().flatten() else {
        dev_log!("Failed to get local storage");
        return;
    };
    
    let config = StoredConfig::new(tonal_center_note, tuning_system, scale);
    
    match serde_json::to_string(&config) {
        Ok(json) => {
            if let Err(_e) = storage.set_item(STORAGE_KEY, &json) {
                dev_log!("Failed to save config to local storage: {:?}", _e);
            }
        }
        Err(_e) => {
            dev_log!("Failed to serialize config: {:?}", _e);
        }
    }
}

pub fn load_config() -> Option<StoredConfig> {
    let window = window()?;
    let storage = window.local_storage().ok().flatten()?;
    let json = storage.get_item(STORAGE_KEY).ok().flatten()?;
    
    match serde_json::from_str::<StoredConfig>(&json) {
        Ok(config) => {
            if config.is_expired() {
                dev_log!("Stored config is expired, using defaults");
                let _ = storage.remove_item(STORAGE_KEY);
                None
            } else {
                dev_log!("Loaded config from local storage: tonal_center={}, tuning_system={:?}, scale={:?}", 
                    config.tonal_center_note, config.tuning_system, config.scale);
                Some(config)
            }
        }
        Err(_e) => {
            dev_log!("Failed to deserialize config: {:?}", _e);
            let _ = storage.remove_item(STORAGE_KEY);
            None
        }
    }
}

pub fn clear_config() {
    if let Some(window) = window() {
        if let Some(storage) = window.local_storage().ok().flatten() {
            let _ = storage.remove_item(STORAGE_KEY);
        }
    }
}

