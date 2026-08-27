use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::core::constants::{
    DEFAULT_GEMINI_MODEL, DEFAULT_OPENROUTER_MODEL, DEFAULT_OPENCODE_MODEL, OPENROUTER_API, OPENCODE_GO_API,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mc_dir: String,
    pub theme: String,
    pub smart_glue: bool,
    pub google_workers: usize,
    pub google_delay_ms: u64,
    
    // Gemini Settings
    pub gemini_api_key: String,
    pub gemini_model: String,
    pub gemini_delay_ms: u64,

    // OpenAI / OpenRouter Settings
    pub openrouter_api_url: String,
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub openrouter_site_url: String,
    pub openrouter_app_name: String,

    // DeepL Settings
    pub deepl_key: String,

    // OpenCode Go Settings
    #[serde(default)]
    pub opencode_api_key: String,
    #[serde(default = "default_opencode_model")]
    pub opencode_model: String,

    // Translation Preferences
    pub target_language: String,
    pub mc_version: String,
    pub output_mode: String,
    pub pack_name: String,
    pub engine: String,
    pub google_mode: String,
    pub ai_mode: String,
    pub process_mode: String,
    pub translate_mods: bool,
    pub translate_books: bool,
    pub translate_quests: bool,
}

fn default_opencode_model() -> String {
    DEFAULT_OPENCODE_MODEL.to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mc_dir: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            theme: "Dark".to_string(),
            smart_glue: true,
            google_workers: 5,
            google_delay_ms: 100,
            
            gemini_api_key: String::new(),
            gemini_model: DEFAULT_GEMINI_MODEL.to_string(),
            gemini_delay_ms: 4100,

            openrouter_api_url: OPENROUTER_API.to_string(),
            openrouter_api_key: String::new(),
            openrouter_model: DEFAULT_OPENROUTER_MODEL.to_string(),
            openrouter_site_url: "https://github.com/modlang/modlang".to_string(),
            openrouter_app_name: "ModLang Minecraft Translator".to_string(),

            deepl_key: String::new(),

            opencode_api_key: String::new(),
            opencode_model: DEFAULT_OPENCODE_MODEL.to_string(),

            target_language: "Русский".to_string(),
            mc_version: "1.20.1".to_string(),
            output_mode: "resourcepack".to_string(),
            pack_name: "ModLang_Pack".to_string(),
            engine: "google".to_string(),
            google_mode: "single".to_string(),
            ai_mode: "safe".to_string(),
            process_mode: "append".to_string(),
            translate_mods: true,
            translate_books: true,
            translate_quests: true,
        }
    }
}

impl AppConfig {
    pub fn load<P: AsRef<Path>>(filepath: P) -> Self {
        let path = filepath.as_ref();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save(path);
        default_cfg
    }

    pub fn save<P: AsRef<Path>>(&self, filepath: P) -> Result<(), String> {
        let path = filepath.as_ref();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json_str = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json_str).map_err(|e| e.to_string())
    }
}
