use serde::{Deserialize, Serialize};

#[allow(dead_code)]
const STORAGE_KEY: &str = "juncto_settings";

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct UserSettings {
    pub display_name: Option<String>,
    pub camera_id: Option<String>,
    pub mic_id: Option<String>,
    pub resolution: Option<String>,
}

pub fn load_settings() -> UserSettings {
    #[cfg(not(target_arch = "wasm32"))]
    return UserSettings::default();

    #[cfg(target_arch = "wasm32")]
    {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return UserSettings::default(),
    };
    let storage = window.local_storage().ok().flatten();

    if let Some(storage) = storage {
        let _key = STORAGE_KEY;
        if let Ok(Some(json)) = storage.get_item(_key) {
            return serde_json::from_str::<UserSettings>(&json).unwrap_or_default();
        }
    }
    UserSettings::default()
    }
}

pub fn save_settings(settings: &UserSettings) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = settings; // suppress unused warning

    #[cfg(target_arch = "wasm32")]
    {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let storage = window.local_storage().ok().flatten();

    if let Some(storage) = storage {
        if let Ok(json) = serde_json::to_string(settings) {
            let _key = STORAGE_KEY;
            let _ = storage.set_item(_key, &json);
        }
    }
    }
}

pub fn update_setting<F>(update_fn: F)
where
    F: FnOnce(&mut UserSettings),
{
    let mut settings = load_settings();
    update_fn(&mut settings);
    save_settings(&settings);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_serialization() {
        let settings = UserSettings {
            display_name: Some("Test User".to_string()),
            camera_id: Some("cam123".to_string()),
            mic_id: None,
            resolution: Some("hd".to_string()),
        };
        let json = serde_json::to_string(&settings).unwrap();
        let decoded: UserSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.display_name, settings.display_name);
        assert_eq!(decoded.camera_id, settings.camera_id);
    }
}
